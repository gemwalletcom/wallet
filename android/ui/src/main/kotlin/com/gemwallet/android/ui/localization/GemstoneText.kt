package com.gemwallet.android.ui.localization

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.duration.formatDuration
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.QRScanType
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.ScanReceiveMode
import com.wallet.core.primitives.TpslType
import com.wallet.core.primitives.TransactionState
import uniffi.gemstone.AutocloseValidation
import uniffi.gemstone.DelegationState
import uniffi.gemstone.FeeOption
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemApprovalValue
import uniffi.gemstone.GemAssetMenuAction
import uniffi.gemstone.GemBalanceResource
import uniffi.gemstone.GemBalanceRowValue
import uniffi.gemstone.GemBannerDescription
import uniffi.gemstone.GemBannerTitle
import uniffi.gemstone.GemCandleTooltipRow
import uniffi.gemstone.GemCurrencySectionKind
import uniffi.gemstone.GemDelegationStatus
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemEmptyStateText
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemFiatTransactionBadge
import uniffi.gemstone.GemHeaderButtonKind
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.GemPositionChange
import uniffi.gemstone.GemRecipientErrorDisplay
import uniffi.gemstone.GemRecipientSectionKind
import uniffi.gemstone.GemSimulationPayloadTitle
import uniffi.gemstone.GemSlippageCheck
import uniffi.gemstone.PaymentStatus
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemTriggerOrder
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.GemVerificationLevel
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletSubtitle
import uniffi.gemstone.LinkType
import uniffi.gemstone.PerpetualMarginType
import uniffi.gemstone.PerpetualProvider
import uniffi.gemstone.StakeProviderType
import uniffi.gemstone.WalletConnectionVerificationStatus
import uniffi.gemstone.verificationLevel
import uniffi.gemstone.PriceChangeCalculator as GemPriceChangeCalculator

fun GemTransactionTitle.string(context: Context): String = when (this) {
    GemTransactionTitle.Received -> context.getString(R.string.transaction_title_received)
    GemTransactionTitle.Sent -> context.getString(R.string.transaction_title_sent)
    GemTransactionTitle.Transfer -> context.getString(R.string.transfer_title)
    GemTransactionTitle.SmartContract -> context.getString(R.string.transfer_smart_contract_title)
    GemTransactionTitle.Swap -> context.getString(R.string.wallet_swap)
    GemTransactionTitle.Approve -> context.getString(R.string.transfer_approve_title)
    GemTransactionTitle.Stake -> context.getString(R.string.transfer_stake_title)
    GemTransactionTitle.Unstake -> context.getString(R.string.transfer_unstake_title)
    GemTransactionTitle.Redelegate -> context.getString(R.string.transfer_redelegate_title)
    GemTransactionTitle.Rewards -> context.getString(R.string.transfer_rewards_title)
    GemTransactionTitle.Withdraw -> context.getString(R.string.transfer_withdraw_title)
    GemTransactionTitle.ActivateAsset -> context.getString(R.string.transfer_activate_asset_title)
    GemTransactionTitle.Freeze -> context.getString(R.string.transfer_freeze_title)
    GemTransactionTitle.Unfreeze -> context.getString(R.string.transfer_unfreeze_title)
    GemTransactionTitle.Earn -> context.getString(R.string.common_earn)
    is GemTransactionTitle.PerpetualOpen -> perpetualTitle(context, direction, R.string.perpetual_open_direction, R.string.perpetual_position)
    is GemTransactionTitle.PerpetualClose -> perpetualTitle(context, direction, R.string.perpetual_close_direction, R.string.perpetual_close_position)
    GemTransactionTitle.PerpetualModify -> context.getString(R.string.perpetual_modify)
}

@StringRes
fun AutocloseValidation.stringRes(): Int? = when (this) {
    AutocloseValidation.VALID -> null
    AutocloseValidation.INVALID_AMOUNT -> R.string.errors_invalid_amount
    AutocloseValidation.TRIGGER_MUST_BE_HIGHER -> R.string.errors_perpetual_trigger_price_higher
    AutocloseValidation.TRIGGER_MUST_BE_LOWER -> R.string.errors_perpetual_trigger_price_lower
}

@StringRes
fun GemFiatTransactionBadge.stringRes(): Int = when (this) {
    GemFiatTransactionBadge.PENDING -> R.string.transaction_status_pending
    GemFiatTransactionBadge.FAILED -> R.string.transaction_status_failed
}

fun GemWalletSubtitle.string(context: Context): String = when (this) {
    GemWalletSubtitle.Multicoin -> context.getString(R.string.wallet_multicoin)
    is GemWalletSubtitle.Address -> value
}

@StringRes
fun DelegationState.stateRes(): Int = when (this) {
    DelegationState.ACTIVE -> R.string.stake_active
    DelegationState.PENDING -> R.string.stake_pending
    DelegationState.INACTIVE -> R.string.stake_inactive
    DelegationState.ACTIVATING -> R.string.stake_activating
    DelegationState.DEACTIVATING -> R.string.stake_deactivating
    DelegationState.AWAITING_WITHDRAWAL -> R.string.stake_awaiting_withdrawal
}

@Composable
fun GemDelegationStatus.stateText(): String = stringResource(
    state.stateRes(),
)

@StringRes
fun GemTransactionFilter.getLabel() = when (this) {
    GemTransactionFilter.TRANSFERS -> R.string.transfer_title
    GemTransactionFilter.SWAPS -> R.string.wallet_swap
    GemTransactionFilter.STAKE -> R.string.wallet_stake
    GemTransactionFilter.SMART_CONTRACT -> R.string.transfer_smart_contract_title
    GemTransactionFilter.PERPETUALS -> R.string.perpetuals_title
    GemTransactionFilter.OTHERS -> R.string.transfer_other_title
}

private fun perpetualTitle(context: Context, direction: uniffi.gemstone.PerpetualDirection?, @StringRes directionTitle: Int, @StringRes fallback: Int): String {
    val side = when (val side = direction?.toPrimitives()) {
        null -> return context.getString(fallback)
        else -> context.getString(side.stringRes())
    }
    return context.getString(directionTitle, side)
}

fun GemBalanceRowValue.text(context: Context): String = when (this) {
    is GemBalanceRowValue.Amount -> amount.text()
    is GemBalanceRowValue.Apr -> context.getString(R.string.stake_apr, apr?.text().orEmpty())
}

fun GemLocalizedText.string(context: Context): String = when (this) {
    is GemLocalizedText.WalletDefaultName -> context.getString(R.string.wallet_default_name, index)

    is GemLocalizedText.WalletDefaultNameChain ->
        context.getString(R.string.wallet_default_name_chain, networkName, index)

    GemLocalizedText.WalletMulticoin -> context.getString(R.string.wallet_multicoin)

    is GemLocalizedText.ChainNetworkName -> chain.requireChain().networkName()

    is GemLocalizedText.DelegationState -> context.getString(state.stateRes())

    is GemLocalizedText.TransactionState -> context.getString(state.toPrimitives().statusLabelRes())

    is GemLocalizedText.Resource -> context.getString(resource.toPrimitives().stringRes())

    is GemLocalizedText.Text -> text

    is GemLocalizedText.Number -> number.text()

    GemLocalizedText.None -> context.getString(R.string.common_none)

    GemLocalizedText.SlippageAuto -> context.getString(R.string.swap_slippage_auto)

    GemLocalizedText.RewardsUnverified -> context.getString(R.string.rewards_unverified_description)

    is GemLocalizedText.RewardsPending -> context.getString(R.string.rewards_pending_description, countdown.formatDuration())

    GemLocalizedText.RewardsPendingReady -> context.getString(R.string.rewards_pending_description_ready)

    GemLocalizedText.ErrorOccurred -> context.getString(R.string.errors_error_occurred)

    GemLocalizedText.UnlimitedApprovalWarning -> context.getString(R.string.simulation_warning_unlimited_token_approval_description)

    GemLocalizedText.ExternallyOwnedSpenderWarning -> context.getString(R.string.simulation_warning_externally_owned_spender_description)

    GemLocalizedText.SuspiciousAddress -> context.getString(R.string.common_suspicious_address)

    GemLocalizedText.InvalidTokenId -> context.getString(R.string.errors_token_invalid_id)

    is GemLocalizedText.TriggerOrder -> GemPerpetual(PerpetualProvider.HYPERCORE).use { it.triggerOrderText(context.getString(order.stringRes()), price?.text()) }

    is GemLocalizedText.Pnl -> GemPriceChangeCalculator().use { it.pnlText(amount.text(), percent.text()) }

    is GemLocalizedText.Margin -> GemPerpetual(PerpetualProvider.HYPERCORE).use { it.marginText(amount.text(), context.getString(marginType.stringRes())) }

    is GemLocalizedText.Position -> GemPerpetual(PerpetualProvider.HYPERCORE).use { it.positionText(context.getString(direction.toPrimitives().stringRes()), leverage) }

    is GemLocalizedText.Apr -> context.getString(R.string.stake_apr, value?.text().orEmpty())

    is GemLocalizedText.PriceImpactWarning -> context.getString(R.string.swap_price_impact_warning_description, percent.text(), symbol)

    is GemLocalizedText.Balance -> context.getString(R.string.transfer_balance, amount.text())

    is GemLocalizedText.StakeProvider -> when (provider) {
        StakeProviderType.STAKE -> context.getString(R.string.transfer_stake_title)
        StakeProviderType.EARN -> context.getString(R.string.common_earn)
    }

    is GemLocalizedText.PositionChange -> when (change) {
        GemPositionChange.INCREASE -> context.getString(R.string.perpetual_increase_direction, context.getString(direction.toPrimitives().stringRes()))
        GemPositionChange.REDUCE -> context.getString(R.string.perpetual_reduce_direction, context.getString(direction.toPrimitives().stringRes()))
    }

    is GemLocalizedText.FeeRate -> when (unit) {
        FeeUnitType.SAT_VB -> "${rate.text()} ${context.getString(R.string.fee_rate_satvB)}"
        FeeUnitType.GWEI -> "${rate.text()} ${context.getString(R.string.fee_rate_gwei)}"
        FeeUnitType.NATIVE -> rate.text()
    }
}

@StringRes
fun ChartPeriod.stringRes(): Int = when (this) {
    ChartPeriod.Hour -> R.string.charts_hour
    ChartPeriod.Day -> R.string.charts_day
    ChartPeriod.Week -> R.string.charts_week
    ChartPeriod.Month -> R.string.charts_month
    ChartPeriod.Year -> R.string.charts_year
    ChartPeriod.All -> R.string.charts_all
}

@StringRes
fun LinkType.stringRes(): Int = when (this) {
    LinkType.X -> R.string.social_x
    LinkType.DISCORD -> R.string.social_discord
    LinkType.REDDIT -> R.string.social_reddit
    LinkType.TELEGRAM -> R.string.social_telegram
    LinkType.GIT_HUB -> R.string.social_github
    LinkType.YOU_TUBE -> R.string.social_youtube
    LinkType.FACEBOOK -> R.string.social_facebook
    LinkType.WEBSITE -> R.string.social_website
    LinkType.COINGECKO -> R.string.social_coingecko
    LinkType.OPEN_SEA -> R.string.social_opensea
    LinkType.INSTAGRAM -> R.string.social_instagram
    LinkType.MAGIC_EDEN -> R.string.social_magiceden
    LinkType.COIN_MARKET_CAP -> R.string.social_coinmarketcap
    LinkType.TIK_TOK -> R.string.social_tiktok
}

@StringRes
fun Resource.stringRes(): Int = when (this) {
    Resource.Bandwidth -> R.string.stake_resource_bandwidth
    Resource.Energy -> R.string.stake_resource_energy
}

@StringRes
fun QRScanType.stringRes(): Int = when (this) {
    QRScanType.Universal -> R.string.wallet_scan_hint
    QRScanType.WalletConnect -> R.string.wallet_connect_title
    QRScanType.Address -> R.string.wallet_scan_hint_address
    QRScanType.Memo -> R.string.transfer_memo
    QRScanType.Url -> R.string.common_url
    QRScanType.TokenContract -> R.string.wallet_import_contract_address_field
    QRScanType.SecretPhrase -> R.string.common_secret_phrase
    QRScanType.PrivateKey -> R.string.common_private_key
}

fun GemSimulationPayloadTitle.text(context: Context): String = when (this) {
    GemSimulationPayloadTitle.Contract -> context.getString(R.string.asset_contract)
    GemSimulationPayloadTitle.Method -> context.getString(R.string.common_method)
    GemSimulationPayloadTitle.Token -> context.getString(R.string.common_token)
    GemSimulationPayloadTitle.Spender -> context.getString(R.string.transfer_to)
    GemSimulationPayloadTitle.Value -> context.getString(R.string.perpetual_value)
    GemSimulationPayloadTitle.Expiration -> context.getString(R.string.common_expiration)
    is GemSimulationPayloadTitle.Custom -> label
}

@StringRes
fun TransactionState.statusLabelRes(): Int = when (this) {
    TransactionState.Pending,
    TransactionState.InTransit,
    -> R.string.transaction_status_pending

    TransactionState.Confirmed -> R.string.transaction_status_confirmed

    TransactionState.Failed -> R.string.transaction_status_failed

    TransactionState.Reverted -> R.string.transaction_status_reverted

    TransactionState.Refunded -> R.string.transaction_status_refunded
}

@StringRes
fun GemTransactionStateTone.infoDescriptionRes(): Int = when (this) {
    GemTransactionStateTone.PENDING -> R.string.info_transaction_pending_description

    GemTransactionStateTone.SUCCESS -> R.string.info_transaction_success_description

    GemTransactionStateTone.ERROR,
    GemTransactionStateTone.REFUNDED,
    -> R.string.info_transaction_error_description
}

@StringRes
fun PerpetualDirection.stringRes(): Int = when (this) {
    PerpetualDirection.Long -> R.string.perpetual_long
    PerpetualDirection.Short -> R.string.perpetual_short
}

@StringRes
fun FeePriority.stringRes(): Int = when (this) {
    FeePriority.Normal -> R.string.fee_rates_normal
    FeePriority.Fast -> R.string.fee_rates_fast
}

fun GemApprovalValue.text(context: Context, symbol: String, formatter: ValueFormatter, asset: Asset): String = when (this) {
    is GemApprovalValue.Exact -> formatter.string(value, asset)
    GemApprovalValue.Unlimited -> context.getString(R.string.simulation_header_unlimited_asset, symbol)
}

@StringRes
fun GemVerificationLevel.stringRes(): Int = when (this) {
    GemVerificationLevel.VERIFIED -> R.string.asset_verification_verified
    GemVerificationLevel.UNVERIFIED -> R.string.asset_verification_unverified
    GemVerificationLevel.SUSPICIOUS -> R.string.asset_verification_suspicious
}

@StringRes
fun WalletConnectionVerificationStatus.titleRes(): Int = verificationLevel(this).stringRes()

@StringRes
fun GemAssetMenuAction.stringRes(): Int = when (this) {
    is GemAssetMenuAction.Pin -> if (isPinned) R.string.common_unpin else R.string.common_pin
    GemAssetMenuAction.Hide -> R.string.common_hide
    GemAssetMenuAction.AddToWallet -> R.string.asset_add_to_wallet
    is GemAssetMenuAction.CopyAddress -> R.string.wallet_copy_address
}

@StringRes
fun ScanReceiveMode.stringRes(): Int = when (this) {
    ScanReceiveMode.Scan -> R.string.wallet_scan
    ScanReceiveMode.Receive -> R.string.wallet_receive
}

@StringRes
fun TpslType.autocloseRes(): Int = when (this) {
    TpslType.TakeProfit -> R.string.perpetual_auto_close_take_profit
    TpslType.StopLoss -> R.string.perpetual_auto_close_stop_loss
}

@StringRes
fun GemWalletSecretKind.stringRes(): Int = when (this) {
    GemWalletSecretKind.PHRASE -> R.string.common_secret_phrase
    GemWalletSecretKind.PRIVATE_KEY -> R.string.common_private_key
}

@StringRes
fun GemTransactionRowSubtitle.prefixRes(): Int? = when (this) {
    is GemTransactionRowSubtitle.ToAddress, is GemTransactionRowSubtitle.ToResource -> R.string.transfer_to
    is GemTransactionRowSubtitle.FromAddress, is GemTransactionRowSubtitle.FromResource -> R.string.transfer_from
    is GemTransactionRowSubtitle.Price -> R.string.asset_price
    GemTransactionRowSubtitle.None -> null
}

fun GemEmptyStateText.text(context: Context, symbol: String): String = when (this) {
    GemEmptyStateText.NFTS_TITLE -> context.getString(R.string.nft_state_empty_title)
    GemEmptyStateText.NFTS_DESCRIPTION -> context.getString(R.string.nft_state_empty_description)
    GemEmptyStateText.PRICE_ALERTS_TITLE -> context.getString(R.string.price_alerts_state_empty_title)
    GemEmptyStateText.PRICE_ALERTS_DESCRIPTION -> context.getString(R.string.price_alerts_state_empty_description)
    GemEmptyStateText.CONTACTS_TITLE -> context.getString(R.string.contacts_state_empty_title)
    GemEmptyStateText.CONTACTS_DESCRIPTION -> context.getString(R.string.contacts_state_empty_description)
    GemEmptyStateText.ASSET_TITLE -> context.getString(R.string.asset_state_empty_title)
    GemEmptyStateText.ASSET_DESCRIPTION -> context.getString(R.string.asset_state_empty_description, symbol)
    GemEmptyStateText.ACTIVITY_TITLE -> context.getString(R.string.activity_state_empty_title)
    GemEmptyStateText.ACTIVITY_DESCRIPTION -> context.getString(R.string.activity_state_empty_description)
    GemEmptyStateText.STAKE_TITLE -> context.getString(R.string.stake_state_empty_title)
    GemEmptyStateText.STAKE_DESCRIPTION -> context.getString(R.string.stake_state_empty_description, symbol)
    GemEmptyStateText.EARN_TITLE -> context.getString(R.string.earn_state_empty_title)
    GemEmptyStateText.EARN_DESCRIPTION -> context.getString(R.string.earn_state_empty_description, symbol)
    GemEmptyStateText.WALLET_CONNECT_TITLE -> context.getString(R.string.wallet_connect_no_active_connections)
    GemEmptyStateText.WALLET_CONNECT_DESCRIPTION -> context.getString(R.string.wallet_connect_state_empty_description)
    GemEmptyStateText.RECENTS_TITLE -> context.getString(R.string.recent_activity_state_empty_title)
    GemEmptyStateText.RECENTS_DESCRIPTION -> context.getString(R.string.recent_activity_state_empty_description)
    GemEmptyStateText.NOTIFICATIONS_TITLE -> context.getString(R.string.notifications_inapp_state_empty_title)
    GemEmptyStateText.NOTIFICATIONS_DESCRIPTION -> context.getString(R.string.notifications_inapp_state_empty_description)
    GemEmptyStateText.WATCH_WALLET_TITLE -> context.getString(R.string.wallet_watch_empty_state_title)
    GemEmptyStateText.WATCH_WALLET_DESCRIPTION -> context.getString(R.string.info_watch_wallet_description)
    GemEmptyStateText.NO_ASSETS_FOUND_TITLE -> context.getString(R.string.assets_no_assets_found)
    GemEmptyStateText.SEARCH_DESCRIPTION -> context.getString(R.string.search_state_empty_description)
    GemEmptyStateText.SEARCH_ASSETS_DESCRIPTION -> context.getString(R.string.assets_state_empty_search_description)
    GemEmptyStateText.SEARCH_ACTIVITY_TITLE -> context.getString(R.string.activity_state_empty_search_title)
    GemEmptyStateText.SEARCH_ACTIVITY_DESCRIPTION -> context.getString(R.string.activity_state_empty_search_description)
    GemEmptyStateText.SEARCH_NETWORKS_TITLE -> context.getString(R.string.networks_state_empty_search_title)
    GemEmptyStateText.SEARCH_PERPETUALS_TITLE -> context.getString(R.string.perpetuals_empty_state_no_markets_found)
}

@StringRes
fun GemEmptyStateAction.title(): Int = when (this) {
    GemEmptyStateAction.BUY -> R.string.wallet_buy
    GemEmptyStateAction.SWAP -> R.string.wallet_swap
    GemEmptyStateAction.RECEIVE -> R.string.wallet_receive
    GemEmptyStateAction.ADD_CUSTOM_TOKEN -> R.string.assets_add_custom_token
    GemEmptyStateAction.MANAGE_TOKEN_LIST -> R.string.wallet_manage_token_list
    GemEmptyStateAction.CLEAR_FILTERS -> R.string.filter_clear
}

fun GemErrorText.text(context: Context): String = when (this) {
    GemErrorText.Cancelled -> context.getString(R.string.errors_cancelled)
    GemErrorText.NetworkOffline -> context.getString(R.string.errors_network_offline)
    is GemErrorText.NetworkMessage -> context.getString(R.string.errors_network_error, text)
    is GemErrorText.NetworkStatus -> context.getString(R.string.errors_network_error, status.toString())
    GemErrorText.InvalidNetworkId -> context.getString(R.string.errors_invalid_network_id)
    GemErrorText.InvalidUrl -> context.getString(R.string.errors_invalid_url)
    GemErrorText.NotSupported -> context.getString(R.string.errors_not_supported)
    GemErrorText.UnsupportedChain -> context.getString(R.string.errors_connections_unsupported_chain)
    GemErrorText.MaliciousOrigin -> context.getString(R.string.errors_connections_malicious_origin)
    GemErrorText.NoSupportedWallets -> context.getString(R.string.errors_connections_no_supported_wallets)
    is GemErrorText.Payment -> status.errorText(context)
    GemErrorText.InvalidSecretPhrase -> context.getString(R.string.errors_import_invalid_secret_phrase)
    is GemErrorText.InvalidSecretPhraseWords -> context.getString(R.string.errors_import_invalid_secret_phrase_word, words.joinToString())
    GemErrorText.InvalidPrivateKey -> context.getString(R.string.errors_import_invalid_private_key)
    GemErrorText.InvalidAddress -> context.getString(R.string.errors_invalid_address_name)
    GemErrorText.NoAccountForChain -> context.getString(R.string.errors_wallet_account_missing)
    GemErrorText.Unknown -> context.getString(R.string.errors_unknown)
    is GemErrorText.Message -> text
}

fun PaymentStatus.errorText(context: Context): String = when (this) {
    PaymentStatus.REQUIRES_ACTION, PaymentStatus.FAILED -> context.getString(R.string.errors_payment_failed)
    PaymentStatus.PROCESSING -> context.getString(R.string.errors_payment_in_progress)
    PaymentStatus.SUCCEEDED -> context.getString(R.string.errors_payment_paid)
    PaymentStatus.EXPIRED -> context.getString(R.string.errors_payment_expired)
    PaymentStatus.CANCELLED -> context.getString(R.string.errors_payment_cancelled)
}

@Composable
fun GemErrorText.text(): String = text(LocalContext.current)

@StringRes
fun GemHeaderButtonKind.stringRes(): Int = when (this) {
    GemHeaderButtonKind.SEND -> R.string.wallet_send
    GemHeaderButtonKind.RECEIVE -> R.string.wallet_receive
    GemHeaderButtonKind.BUY -> R.string.wallet_buy
    GemHeaderButtonKind.SWAP -> R.string.wallet_swap
    GemHeaderButtonKind.DEPOSIT -> R.string.wallet_deposit
    GemHeaderButtonKind.WITHDRAW -> R.string.wallet_withdraw
    GemHeaderButtonKind.MORE -> R.string.wallet_more
}

@StringRes
fun GemCandleTooltipRow.stringRes(): Int = when (this) {
    GemCandleTooltipRow.OPEN -> R.string.charts_price_open
    GemCandleTooltipRow.HIGH -> R.string.charts_price_high
    GemCandleTooltipRow.LOW -> R.string.charts_price_low
    GemCandleTooltipRow.CLOSE -> R.string.charts_price_close
    GemCandleTooltipRow.CHANGE -> R.string.charts_price_change
    GemCandleTooltipRow.VOLUME -> R.string.perpetual_volume
}

@StringRes
fun GemCurrencySectionKind.stringRes(): Int = when (this) {
    GemCurrencySectionKind.RECOMMENDED -> R.string.common_recommended
    GemCurrencySectionKind.ALL -> R.string.common_all
}

@StringRes
fun GemRecipientSectionKind.stringRes(): Int = when (this) {
    GemRecipientSectionKind.PINNED -> R.string.common_pinned
    GemRecipientSectionKind.CONTACTS -> R.string.contacts_title
    GemRecipientSectionKind.WALLETS -> R.string.transfer_recipient_my_wallets
    GemRecipientSectionKind.VIEW_WALLETS -> R.string.transfer_recipient_view_wallets
}

fun GemBalanceResource.titleRes(): Int = when (this) {
    GemBalanceResource.ENERGY -> R.string.stake_resource_energy
    GemBalanceResource.BANDWIDTH -> R.string.stake_resource_bandwidth
}

fun GemTransactionRowSubtitle.text(context: Context): String? = when (this) {
    is GemTransactionRowSubtitle.ToAddress -> prefixed(context, prefixRes(), participant)
    is GemTransactionRowSubtitle.FromAddress -> prefixed(context, prefixRes(), participant)
    is GemTransactionRowSubtitle.ToResource -> prefixed(context, prefixRes(), context.getString(resource.toPrimitives().stringRes()))
    is GemTransactionRowSubtitle.FromResource -> prefixed(context, prefixRes(), context.getString(resource.toPrimitives().stringRes()))
    is GemTransactionRowSubtitle.Price -> prefixRes()?.let { "${context.getString(it)}: ${price.text()}" }
    GemTransactionRowSubtitle.None -> null
}

private fun prefixed(context: Context, @StringRes prefix: Int?, value: String): String? = prefix?.let { res -> value.takeIf { it.isNotEmpty() }?.let { "${context.getString(res)} $it" } }

@StringRes
fun FeeOption.stringRes(): Int = when (this) {
    FeeOption.TOKEN_ACCOUNT_CREATION -> R.string.banner_account_activation_title
}

fun bannerTitle(context: Context, title: GemBannerTitle): String = when (title) {
    is GemBannerTitle.Stake -> context.getString(R.string.banner_stake_title, title.assetName)
    GemBannerTitle.AccountActivation -> context.getString(R.string.banner_account_activation_title)
    GemBannerTitle.Warning -> context.getString(R.string.common_warning)
    GemBannerTitle.ActivateAsset -> context.getString(R.string.transfer_activate_asset_title)
    GemBannerTitle.SuspiciousAsset -> context.getString(R.string.banner_asset_status_title)
    GemBannerTitle.Onboarding -> context.getString(R.string.banner_onboarding_title)
    GemBannerTitle.TradePerpetuals -> context.getString(R.string.banner_perpetuals_title)
}

fun bannerDescription(context: Context, description: GemBannerDescription): String = when (description) {
    is GemBannerDescription.Stake -> context.getString(R.string.banner_stake_description, description.assetSymbol)

    is GemBannerDescription.AccountActivation -> context.getString(
        R.string.banner_account_activation_description,
        description.networkName,
        description.fee.text(),
    )

    is GemBannerDescription.ExternallyControlledAccount -> context.getString(R.string.warnings_externally_controlled_account, description.networkName)

    is GemBannerDescription.ActivateAsset -> context.getString(
        R.string.banner_activate_asset_description,
        description.assetSymbol,
        description.networkName,
    )

    GemBannerDescription.SuspiciousAsset -> context.getString(R.string.banner_asset_status_description)

    GemBannerDescription.Onboarding -> context.getString(R.string.banner_onboarding_description)

    GemBannerDescription.TradePerpetuals -> context.getString(R.string.banner_perpetuals_description)
}

fun GemRecipientErrorDisplay.string(context: Context): String = when (this) {
    is GemRecipientErrorDisplay.InvalidAddress -> context.getString(R.string.errors_invalid_asset_address, network)
}

@StringRes
fun GemListSectionTitle.titleRes(): Int? = when (this) {
    GemListSectionTitle.NONE -> null
    GemListSectionTitle.BALANCES -> R.string.asset_balances
    GemListSectionTitle.COMMUNITY -> R.string.settings_community
    GemListSectionTitle.MANAGE -> R.string.common_manage
    GemListSectionTitle.RESOURCES -> R.string.asset_resources
    GemListSectionTitle.SOCIAL_LINKS -> R.string.social_links
}

fun GemListRowTitle.text(context: Context): String = when (this) {
    GemListRowTitle.API -> "API"
    GemListRowTitle.STREAM -> "Stream"
    GemListRowTitle.GEM_WALLET_NODE -> context.getString(R.string.nodes_gem_wallet_node)
    GemListRowTitle.NAME -> context.getString(R.string.asset_name)
    GemListRowTitle.NETWORK -> context.getString(R.string.transfer_network)
    GemListRowTitle.ADDRESS -> context.getString(R.string.common_address)
    GemListRowTitle.AVAILABLE -> context.getString(R.string.asset_balances_available)
    GemListRowTitle.STAKE -> context.getString(R.string.wallet_stake)
    GemListRowTitle.EARN -> context.getString(R.string.common_earn)
    GemListRowTitle.PENDING_UNCONFIRMED -> context.getString(R.string.stake_pending)
    GemListRowTitle.RESERVED -> context.getString(R.string.asset_balances_reserved)
    GemListRowTitle.ERROR -> context.getString(R.string.errors_error_occurred)
    GemListRowTitle.TERMS_OF_SERVICE -> context.getString(R.string.settings_terms_of_services)
    GemListRowTitle.PRIVACY_POLICY -> context.getString(R.string.settings_privacy_policy)
    GemListRowTitle.WEBSITE -> context.getString(R.string.settings_website)
    GemListRowTitle.VERSION -> context.getString(R.string.settings_version)
    GemListRowTitle.UPDATE_APP -> context.getString(R.string.update_app_title)
    GemListRowTitle.WALLETS -> context.getString(R.string.wallets_title)
    GemListRowTitle.SECURITY -> context.getString(R.string.settings_security)
    GemListRowTitle.NOTIFICATIONS -> context.getString(R.string.settings_notifications_title)
    GemListRowTitle.PREFERENCES -> context.getString(R.string.settings_preferences_title)
    GemListRowTitle.WALLET_CONNECT -> context.getString(R.string.wallet_connect_title)
    GemListRowTitle.SUPPORT -> context.getString(R.string.settings_support)
    GemListRowTitle.REWARDS -> context.getString(R.string.rewards_title)
    GemListRowTitle.MY_REFERRAL_CODE -> context.getString(R.string.rewards_my_referral_code)
    GemListRowTitle.REFERRALS -> context.getString(R.string.rewards_referrals)
    GemListRowTitle.POINTS -> context.getString(R.string.rewards_points)
    GemListRowTitle.INVITED_BY -> context.getString(R.string.rewards_invited_by)
    GemListRowTitle.ABOUT_US -> context.getString(R.string.settings_aboutus)
    GemListRowTitle.DEVELOPER -> context.getString(R.string.settings_developer)
    GemListRowTitle.AUTHENTICATION -> context.getString(R.string.settings_enable_passcode)
    GemListRowTitle.LOCK_PERIOD -> context.getString(R.string.lock_require_authentication)
    GemListRowTitle.PRIVACY_LOCK -> context.getString(R.string.lock_privacy_lock)
    GemListRowTitle.HIDE_BALANCE -> context.getString(R.string.settings_hide_balance)
    GemListRowTitle.CURRENCY -> context.getString(R.string.settings_currency)
    GemListRowTitle.LANGUAGE -> context.getString(R.string.settings_language)
    GemListRowTitle.APPEARANCE -> context.getString(R.string.settings_appearance_title)
    GemListRowTitle.NETWORKS -> context.getString(R.string.settings_networks_title)
    GemListRowTitle.CONTACTS -> context.getString(R.string.contacts_title)
    GemListRowTitle.PERPETUALS -> context.getString(R.string.perpetuals_title)
    GemListRowTitle.PERPETUAL_LEVERAGE -> context.getString(R.string.settings_preferences_perpetual_default_leverage)
    GemListRowTitle.PERPETUAL_TAKE_PROFIT -> context.getString(R.string.settings_preferences_perpetual_default_take_profit)
    GemListRowTitle.PERPETUAL_STOP_LOSS -> context.getString(R.string.settings_preferences_perpetual_default_stop_loss)
    GemListRowTitle.DAILY_VOLUME -> context.getString(R.string.markets_daily_volume)
    GemListRowTitle.OPEN_INTEREST -> context.getString(R.string.info_perpetual_open_interest_title)
    GemListRowTitle.FUNDING_APR -> context.getString(R.string.info_perpetual_funding_apr_title)
    GemListRowTitle.STAKE_APR -> context.getString(R.string.stake_apr, "")
    GemListRowTitle.LOCK_TIME -> context.getString(R.string.stake_lock_time)
    GemListRowTitle.MINIMUM_AMOUNT -> context.getString(R.string.stake_minimum_amount)
    GemListRowTitle.NETWORK_FEE -> context.getString(R.string.transfer_network_fee)
    GemListRowTitle.VALIDATOR -> context.getString(R.string.stake_validator)
    GemListRowTitle.PROVIDER -> context.getString(R.string.common_provider)
    GemListRowTitle.STATUS -> context.getString(R.string.transaction_status)
    GemListRowTitle.ACTIVE_IN -> context.getString(R.string.stake_active_in)
    GemListRowTitle.AVAILABLE_IN -> context.getString(R.string.stake_available_in)
    GemListRowTitle.DATE -> context.getString(R.string.transaction_date)
    GemListRowTitle.RESOURCE -> context.getString(R.string.stake_resource)
    GemListRowTitle.REWARDS_UNVERIFIED -> context.getString(R.string.rewards_unverified_title)
    GemListRowTitle.REWARDS_PENDING -> context.getString(R.string.rewards_pending_title)
    GemListRowTitle.WARNING -> context.getString(R.string.common_warning)
    GemListRowTitle.UNLIMITED_APPROVAL -> context.getString(R.string.simulation_warning_unlimited_token_approval_title)
    GemListRowTitle.NFT_COLLECTION_APPROVAL -> context.getString(R.string.simulation_warning_nft_collection_approval_title)
    GemListRowTitle.SYMBOL -> context.getString(R.string.asset_symbol)
    GemListRowTitle.DECIMALS -> context.getString(R.string.asset_decimals)
    GemListRowTitle.TYPE -> context.getString(R.string.common_type)
    GemListRowTitle.AUTO_CLOSE -> context.getString(R.string.perpetual_auto_close)
    GemListRowTitle.SIZE -> context.getString(R.string.perpetual_size)
    GemListRowTitle.POSITION -> context.getString(R.string.perpetual_position)
    GemListRowTitle.DETAILS -> context.getString(R.string.common_details)
    GemListRowTitle.SLIPPAGE -> context.getString(R.string.swap_slippage)
    GemListRowTitle.UNREALIZED_PNL -> context.getString(R.string.perpetual_unrealized_pnl)
    GemListRowTitle.ACCOUNT_LEVERAGE -> context.getString(R.string.perpetual_account_leverage)
    GemListRowTitle.MARGIN_USAGE -> context.getString(R.string.perpetual_margin_usage)
    GemListRowTitle.ALL_TIME_PNL -> context.getString(R.string.perpetual_all_time_pnl)
    GemListRowTitle.VOLUME -> context.getString(R.string.perpetual_volume)
    GemListRowTitle.PRICE_IMPACT -> context.getString(R.string.swap_price_impact)
    GemListRowTitle.MINIMUM_RECEIVE -> context.getString(R.string.swap_min_receive)
    GemListRowTitle.ESTIMATED_TIME -> context.getString(R.string.swap_estimated_time_title)
    GemListRowTitle.MARKET_PRICE -> context.getString(R.string.perpetual_market_price)
    GemListRowTitle.ENTRY_PRICE -> context.getString(R.string.perpetual_entry_price)
    GemListRowTitle.LIQUIDATION_PRICE -> context.getString(R.string.info_perpetual_liquidation_price_title)
    GemListRowTitle.MARGIN -> context.getString(R.string.perpetual_margin)
    GemListRowTitle.FUNDING_PAYMENTS -> context.getString(R.string.info_perpetual_funding_payments_title)
    GemListRowTitle.MARKET_CAP -> context.getString(R.string.asset_market_cap)
    GemListRowTitle.FULLY_DILUTED_VALUATION -> context.getString(R.string.info_fully_diluted_valuation_title)
    GemListRowTitle.TRADING_VOLUME -> context.getString(R.string.asset_trading_volume)
    GemListRowTitle.CIRCULATING_SUPPLY -> context.getString(R.string.asset_circulating_supply)
    GemListRowTitle.TOTAL_SUPPLY -> context.getString(R.string.asset_total_supply)
    GemListRowTitle.MAX_SUPPLY -> context.getString(R.string.info_max_supply_title)
    GemListRowTitle.ALL_TIME_HIGH -> context.getString(R.string.asset_all_time_high)
    GemListRowTitle.ALL_TIME_LOW -> context.getString(R.string.asset_all_time_low)
    GemListRowTitle.WALLET -> context.getString(R.string.common_wallet)
    GemListRowTitle.CONTRACT -> context.getString(R.string.asset_contract)
    GemListRowTitle.TOKEN_ID -> context.getString(R.string.asset_token_id)
    GemListRowTitle.COLLECTION -> context.getString(R.string.nft_collection)
    GemListRowTitle.PRICE -> context.getString(R.string.asset_price)
    GemListRowTitle.PNL -> context.getString(R.string.perpetual_pnl)
    GemListRowTitle.PIN -> context.getString(R.string.common_pin)
    GemListRowTitle.UNPIN -> context.getString(R.string.common_unpin)
    GemListRowTitle.ADD_TO_WALLET -> context.getString(R.string.asset_add_to_wallet)
    GemListRowTitle.PRICE_ALERTS -> context.getString(R.string.settings_price_alerts_title)
    GemListRowTitle.SET_PRICE_ALERT -> context.getString(R.string.price_alerts_set_alert_title)
    GemListRowTitle.ENERGY -> context.getString(R.string.stake_resource_energy)
    GemListRowTitle.BANDWIDTH -> context.getString(R.string.stake_resource_bandwidth)
}

fun GemSlippageCheck.footerText(context: Context, minimumText: String, maximumText: String): String? = when (this) {
    GemSlippageCheck.BELOW_MINIMUM -> context.getString(R.string.common_minimum_value, minimumText)
    GemSlippageCheck.ABOVE_MAXIMUM -> context.getString(R.string.common_maximum_value, maximumText)
    GemSlippageCheck.HIGH -> context.getString(R.string.swap_slippage_warning)
    GemSlippageCheck.VALID -> null
}

@StringRes
fun GemTriggerOrder.stringRes(): Int = when (this) {
    GemTriggerOrder.TAKE_PROFIT -> R.string.perpetual_take_profit
    GemTriggerOrder.STOP_LOSS -> R.string.perpetual_stop_loss
}

@StringRes
fun PerpetualMarginType.stringRes(): Int = when (this) {
    PerpetualMarginType.CROSS -> R.string.perpetual_margin_cross
    PerpetualMarginType.ISOLATED -> R.string.perpetual_margin_isolated
}

fun GemLatencyStatus.text(context: Context): String = when (this) {
    is GemLatencyStatus.Loading -> ""
    is GemLatencyStatus.Error -> context.getString(R.string.errors_error)
    is GemLatencyStatus.Result -> context.getString(R.string.common_latency_in_ms, latency.value.toLong())
}
