use async_trait::async_trait;
use gem_client::Client;
use helius::{NativeAmount, TokenAmount, Transaction};
use primitives::{
    AssetId, Chain, SwapProvider,
    contract_constants::SOLANA_WRAPPED_SOL_TOKEN_ADDRESS,
    swap::{SwapPartnerTransaction, SwapReferralFee, SwapStatus},
};

use crate::{
    SwapperError,
    fees::default_referral_fees,
    partner::{SwapPartnerCursor, SwapPartnerProvider, SwapPartnerTransactionsPage},
};

const TRANSACTIONS_LIMIT: usize = 100;

pub struct SolanaPartnerProvider<C: Client> {
    client: helius::Client<C>,
    fee_address: String,
}

impl<C: Client> SolanaPartnerProvider<C> {
    pub fn new(client: C) -> Self {
        Self {
            client: helius::Client::new(client),
            fee_address: default_referral_fees().solana.address,
        }
    }
}

fn map_asset_id(mint: &str) -> AssetId {
    if mint == SOLANA_WRAPPED_SOL_TOKEN_ADDRESS {
        return AssetId::from_chain(Chain::Solana);
    }
    AssetId::from_token(Chain::Solana, mint)
}

fn map_balance_changes(transaction: &Transaction, address: &str) -> Vec<(AssetId, i128)> {
    let owned_accounts = || {
        transaction
            .account_data
            .iter()
            .filter(|account| account.account == address || account.token_balance_changes.iter().any(|change| change.user_account == address))
    };
    let tokens = owned_accounts()
        .flat_map(|account| &account.token_balance_changes)
        .filter(|change| change.user_account == address)
        .filter_map(|change| Some((map_asset_id(&change.mint), change.raw_token_amount.token_amount.parse::<i128>().ok()?)));
    let network_fee = if address == transaction.fee_payer { i128::from(transaction.fee) } else { 0 };
    let native = owned_accounts().map(|account| i128::from(account.native_balance_change)).sum::<i128>() + network_fee;
    tokens.chain([(AssetId::from_chain(Chain::Solana), native)]).filter(|(_, amount)| *amount != 0).collect()
}

fn map_event_amount(native: Option<&NativeAmount>, tokens: &[TokenAmount]) -> Option<(AssetId, i128)> {
    if let Some(native) = native {
        return Some((AssetId::from_chain(Chain::Solana), native.amount.parse().ok()?));
    }
    let first = tokens.first()?;
    let amount = tokens
        .iter()
        .filter(|token| token.mint == first.mint)
        .map(|token| token.raw_token_amount.token_amount.parse::<i128>().ok())
        .sum::<Option<i128>>()?;
    Some((map_asset_id(&first.mint), amount))
}

fn map_swap_legs(transaction: &Transaction) -> Option<((AssetId, i128), (AssetId, i128))> {
    let changes = map_balance_changes(transaction, &transaction.fee_payer);
    if let (Some((from_asset_id, from_amount)), Some(to)) = (changes.iter().find(|(_, amount)| *amount < 0), changes.iter().find(|(_, amount)| *amount > 0)) {
        return Some(((from_asset_id.clone(), -from_amount), to.clone()));
    }
    let swap = transaction.events.swap.as_ref()?;
    Some((map_event_amount(swap.native_input.as_ref(), &swap.token_inputs)?, map_event_amount(swap.native_output.as_ref(), &swap.token_outputs)?))
}

pub fn map_partner_transaction(transaction: &Transaction, fee_address: &str) -> Option<SwapPartnerTransaction> {
    let provider = match transaction.source.as_str() {
        "JUPITER" => SwapProvider::Jupiter,
        "OKX_DEX_ROUTER" => SwapProvider::Okx,
        _ => return None,
    };
    let ((from_asset_id, from_amount), (to_asset_id, to_amount)) = map_swap_legs(transaction)?;
    let referral_fee = map_balance_changes(transaction, fee_address).into_iter().find(|(_, amount)| *amount > 0).map(|(asset_id, amount)| SwapReferralFee {
        asset_id,
        value: amount.to_string(),
        amount_usd: None,
    });
    Some(SwapPartnerTransaction {
        provider,
        provider_transaction_id: transaction.signature.clone(),
        status: if transaction.transaction_error.is_none() { SwapStatus::Completed } else { SwapStatus::Failed },
        from_address: transaction.fee_payer.clone(),
        to_address: transaction.fee_payer.clone(),
        from_asset_id,
        from_value: from_amount.to_string(),
        from_amount_usd: None,
        to_asset_id,
        to_value: to_amount.to_string(),
        to_amount_usd: None,
        referral_fee,
        from_transaction_hash: Some(transaction.signature.clone()),
        to_transaction_hash: Some(transaction.signature.clone()),
    })
}

fn map_next_cursor(cursor: Option<String>, transactions: &[Transaction]) -> SwapPartnerCursor {
    match transactions.last() {
        Some(last) if transactions.len() == TRANSACTIONS_LIMIT => SwapPartnerCursor::Next(last.signature.clone()),
        Some(last) => SwapPartnerCursor::Latest(last.signature.clone()),
        None => SwapPartnerCursor::Latest(cursor.unwrap_or_default()),
    }
}

#[async_trait]
impl<C: Client> SwapPartnerProvider for SolanaPartnerProvider<C> {
    fn name(&self) -> &'static str {
        "solana"
    }

    async fn get_transactions(&self, cursor: Option<String>) -> Result<SwapPartnerTransactionsPage, SwapperError> {
        let cursor = cursor.filter(|cursor| !cursor.is_empty());
        let transactions = self.client.get_address_transactions(&self.fee_address, cursor.clone(), TRANSACTIONS_LIMIT).await?;
        Ok(SwapPartnerTransactionsPage {
            transactions: transactions.iter().filter_map(|transaction| map_partner_transaction(transaction, &self.fee_address)).collect(),
            cursor: map_next_cursor(cursor, &transactions),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEE_ADDRESS: &str = "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy";

    fn transactions() -> Vec<Transaction> {
        serde_json::from_str(include_str!("testdata/helius_transactions.json")).unwrap()
    }

    fn summary(transaction: &Transaction) -> (SwapProvider, AssetId, String, AssetId, String, AssetId, String) {
        let transaction = map_partner_transaction(transaction, FEE_ADDRESS).unwrap();
        let fee = transaction.referral_fee.unwrap();
        (transaction.provider, transaction.from_asset_id, transaction.from_value, transaction.to_asset_id, transaction.to_value, fee.asset_id, fee.value)
    }

    #[test]
    fn test_map_partner_transaction() {
        let transactions = transactions();
        let sol = AssetId::from_chain(Chain::Solana);
        let token = |mint: &str| AssetId::from_token(Chain::Solana, mint);
        let usdc = token("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

        assert_eq!(
            summary(&transactions[0]),
            (
                SwapProvider::Jupiter,
                sol.clone(),
                "2000000".into(),
                token("92xV4U2PHikZFJWGyBvdBXx5CcGTv3MPNAc6YnFNGFqj"),
                "91756992".into(),
                sol.clone(),
                "10000".into()
            )
        );
        assert_eq!(
            summary(&transactions[1]),
            (
                SwapProvider::Jupiter,
                token("LinkhB3afbBKb2EQQu7s7umdZceV3wcvAUJhQAfQ23L"),
                "3000000000".into(),
                usdc.clone(),
                "44913642".into(),
                usdc.clone(),
                "225696".into()
            )
        );
        assert_eq!(
            summary(&transactions[2]),
            (
                SwapProvider::Okx,
                token("HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3"),
                "370000000".into(),
                usdc.clone(),
                "29454974".into(),
                usdc,
                "166111".into()
            )
        );
        assert_eq!(
            summary(&transactions[3]),
            (
                SwapProvider::Okx,
                sol.clone(),
                "24303352".into(),
                token("26f12PmBk77wQV1TzLe8XKkNBvMFggbuypxdtMLzNLzz"),
                "4999981887".into(),
                sol.clone(),
                "136099".into()
            )
        );

        assert_eq!(
            summary(&transactions[4]),
            (
                SwapProvider::Jupiter,
                token("CASHx9KJUStyftLFWGvEVf59SGeG9sh5FfcnZMVPCASH"),
                "100000".into(),
                sol.clone(),
                "841286".into(),
                sol.clone(),
                "4206".into()
            )
        );

        let mut cross_chain = transactions[0].clone();
        cross_chain.source = "MAYAN_SWIFT_BRIDGE".to_string();
        assert_eq!(map_partner_transaction(&cross_chain, FEE_ADDRESS), None);
    }

    #[test]
    fn test_map_next_cursor() {
        let transactions = transactions();
        let last = transactions.last().unwrap().signature.clone();

        assert_eq!(map_next_cursor(None, &transactions), SwapPartnerCursor::Latest(last));
        assert_eq!(map_next_cursor(Some("previous".to_string()), &[]), SwapPartnerCursor::Latest("previous".to_string()));
    }
}
