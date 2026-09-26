package com.gemwallet.android.ui.localization

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.domains.duration.formatDuration
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.boldMarkdown
import com.gemwallet.android.ext.errorTextOrNull
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.perpetual.title
import com.wallet.core.primitives.Appearance
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ConnectionStatus
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.FeeUnitType
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PortfolioType
import com.wallet.core.primitives.QRScanType
import com.wallet.core.primitives.ReportReason
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.ScanReceiveMode
import com.wallet.core.primitives.TpslType
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.WalletSource
import uniffi.gemstone.AddressType
import uniffi.gemstone.AutocloseValidation
import uniffi.gemstone.DelegationState
import uniffi.gemstone.FeeOption
import uniffi.gemstone.GemAcceptTermsItem
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemAmountErrorDisplay
import uniffi.gemstone.GemAmountTitle
import uniffi.gemstone.GemAssetMenuAction
import uniffi.gemstone.GemBalanceRowValue
import uniffi.gemstone.GemBannerButton
import uniffi.gemstone.GemBannerDescription
import uniffi.gemstone.GemBannerTitle
import uniffi.gemstone.GemCandleTooltipRow
import uniffi.gemstone.GemChainsFilterSummary
import uniffi.gemstone.GemCollectibleAction
import uniffi.gemstone.GemConfirmButtonKind
import uniffi.gemstone.GemConfirmDestination
import uniffi.gemstone.GemConfirmErrorDisplay
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmTitle
import uniffi.gemstone.GemCurrencySectionKind
import uniffi.gemstone.GemCustomFeeCheck
import uniffi.gemstone.GemDelegationAction
import uniffi.gemstone.GemDelegationStatus
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemEmptyStateText
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemFiatAmountError
import uniffi.gemstone.GemFiatButtonAction
import uniffi.gemstone.GemFiatQuotePhase
import uniffi.gemstone.GemFiatQuotesMessage
import uniffi.gemstone.GemFiatTransactionBadge
import uniffi.gemstone.GemFiatViewState
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemHeaderButtonKind
import uniffi.gemstone.GemInfoAction
import uniffi.gemstone.GemInfoAmount
import uniffi.gemstone.GemInfoDescription
import uniffi.gemstone.GemInfoTitle
import uniffi.gemstone.GemLatencyStatus
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemLockPeriod
import uniffi.gemstone.GemNameIndicator
import uniffi.gemstone.GemNftList
import uniffi.gemstone.GemNodeCheckRow
import uniffi.gemstone.GemNodeRowTitle
import uniffi.gemstone.GemNodeSubtitle
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemPerpetualConfirmedAction
import uniffi.gemstone.GemPerpetualMarketSection
import uniffi.gemstone.GemPerpetualSection
import uniffi.gemstone.GemPositionChange
import uniffi.gemstone.GemPriceAlertLabel
import uniffi.gemstone.GemPriceAlertPrompt
import uniffi.gemstone.GemPriceAlertSectionKind
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.GemReceiveWarning
import uniffi.gemstone.GemRecipientErrorDisplay
import uniffi.gemstone.GemRecipientSectionKind
import uniffi.gemstone.GemRowText
import uniffi.gemstone.GemSecurityReminderItem
import uniffi.gemstone.GemSelectAssetTitle
import uniffi.gemstone.GemSimulationPayloadTitle
import uniffi.gemstone.GemSlippageFooter
import uniffi.gemstone.GemStakeSection
import uniffi.gemstone.GemSubmitMessage
import uniffi.gemstone.GemSwapButtonAction
import uniffi.gemstone.GemSwapErrorDisplay
import uniffi.gemstone.GemSwapProgressStep
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.GemTransactionParticipantRole
import uniffi.gemstone.GemTransactionRowSubtitle
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemTransactionTitle
import uniffi.gemstone.GemTransactionsFilterSummary
import uniffi.gemstone.GemTriggerOrder
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.GemVerificationLevel
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletSubtitle
import uniffi.gemstone.LinkType
import uniffi.gemstone.PaymentStatus
import uniffi.gemstone.PerpetualMarginType
import uniffi.gemstone.PortfolioChartType
import uniffi.gemstone.StakeProviderType
import uniffi.gemstone.WalletConnectionVerificationStatus
import uniffi.gemstone.verificationLevel
import uniffi.gemstone.PriceChangeCalculator as GemPriceChangeCalculator

private const val EMPTY_VALUE = "-"

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

    is GemLocalizedText.RowTitle -> title.text(context)

    is GemLocalizedText.EnableValue -> context.getString(R.string.settings_enable_value, value)

    is GemLocalizedText.ViewOn -> context.getString(R.string.transaction_view_on, name)

    is GemLocalizedText.ParticipantRole -> context.getString(role.stringRes())

    is GemLocalizedText.ConfirmDestination -> context.getString(destination.title())

    is GemLocalizedText.Number -> number.text()

    GemLocalizedText.None -> context.getString(R.string.common_none)

    GemLocalizedText.SlippageAuto -> context.getString(R.string.swap_slippage_auto)

    GemLocalizedText.RewardsUnverified -> context.getString(R.string.rewards_unverified_description)

    is GemLocalizedText.RewardsPending -> context.getString(R.string.rewards_pending_description, countdown.formatDuration())

    GemLocalizedText.RewardsPendingReady -> context.getString(R.string.rewards_pending_description_ready)

    GemLocalizedText.ErrorOccurred -> context.getString(R.string.errors_error_occurred)

    GemLocalizedText.UnlimitedApprovalWarning -> context.getString(R.string.simulation_warning_unlimited_token_approval_description)

    GemLocalizedText.ExternallyOwnedSpenderWarning -> context.getString(R.string.simulation_warning_externally_owned_spender_description)

    GemLocalizedText.SuspiciousAddressDescription -> context.getString(R.string.common_suspicious_address_description)

    is GemLocalizedText.AddressType -> context.getString(addressType.stringRes())

    GemLocalizedText.InvalidTokenId -> context.getString(R.string.errors_token_invalid_id)

    is GemLocalizedText.TriggerOrder -> "${context.getString(order.stringRes())}: ${price?.text() ?: EMPTY_VALUE}"

    is GemLocalizedText.Pnl -> GemPriceChangeCalculator().use { it.pnlText(amount.text(), percent.text()) }

    is GemLocalizedText.Margin -> "${amount.text()} (${context.getString(marginType.stringRes())})"

    is GemLocalizedText.Position -> "${context.getString(direction.toPrimitives().stringRes()).uppercase()} ${leverage.text()}"

    is GemLocalizedText.PriceAlertLabel -> context.getString(label.stringRes())

    is GemLocalizedText.Apr -> context.getString(R.string.stake_apr, value?.text().orEmpty())

    is GemLocalizedText.PriceImpactWarning -> context.getString(R.string.swap_price_impact_warning_description, percent.text(), symbol)

    is GemLocalizedText.Balance -> context.getString(R.string.transfer_balance, amount.text())

    is GemLocalizedText.AvailableBalance -> context.getString(R.string.wallet_available_balance, amount.text())

    is GemLocalizedText.UnlimitedAsset -> context.getString(R.string.simulation_header_unlimited_asset, symbol)

    GemLocalizedText.NftCollections -> context.getString(R.string.nft_collections)

    GemLocalizedText.NftUnverified -> context.getString(R.string.asset_verification_unverified)

    is GemLocalizedText.RewardsRedeemAsset -> context.getString(R.string.rewards_ways_spend_asset_title, value.text())

    is GemLocalizedText.PriceAlertAddedPriceOver -> context.getString(R.string.price_alerts_added_price_over, value.text())

    is GemLocalizedText.PriceAlertAddedPriceUnder -> context.getString(R.string.price_alerts_added_price_under, value.text())

    is GemLocalizedText.PriceAlertAddedIncreasesBy -> context.getString(R.string.price_alerts_added_increases_by, value.text())

    is GemLocalizedText.PriceAlertAddedDecreasesBy -> context.getString(R.string.price_alerts_added_decreases_by, value.text())

    is GemLocalizedText.SignInWith -> context.getString(R.string.common_sign_in_with, chain.requireChain().networkName())

    GemLocalizedText.ReviewRequest -> context.getString(R.string.transfer_review_request)

    GemLocalizedText.EnableDeveloper -> context.getString(R.string.settings_enable_value, context.getString(R.string.settings_developer))

    GemLocalizedText.DisableDeveloper -> context.getString(R.string.settings_disable_value, context.getString(R.string.settings_developer))

    is GemLocalizedText.StakeProvider -> when (provider) {
        StakeProviderType.STAKE -> context.getString(R.string.transfer_stake_title)
        StakeProviderType.EARN -> context.getString(R.string.common_earn)
    }

    is GemLocalizedText.PositionChange -> when (change) {
        GemPositionChange.INCREASE -> context.getString(R.string.perpetual_increase_direction, context.getString(direction.toPrimitives().stringRes()))
        GemPositionChange.REDUCE -> context.getString(R.string.perpetual_reduce_direction, context.getString(direction.toPrimitives().stringRes()))
    }

    is GemLocalizedText.PerpetualConfirmed -> when (val action = action) {
        is GemPerpetualConfirmedAction.Open -> context.getString(R.string.perpetual_open_direction, context.getString(action.direction.toPrimitives().stringRes()))
        GemPerpetualConfirmedAction.Close -> context.getString(R.string.perpetual_close_position)
        GemPerpetualConfirmedAction.Modify -> context.getString(R.string.perpetual_modify_position)
        GemPerpetualConfirmedAction.Increase -> context.getString(R.string.perpetual_increase_position)
        GemPerpetualConfirmedAction.Reduce -> context.getString(R.string.perpetual_reduce_position)
    }

    is GemLocalizedText.FeeRate -> when (unit) {
        uniffi.gemstone.FeeUnitType.SAT_VB -> "${rate.text()} ${context.getString(R.string.fee_rate_satvB)}"
        uniffi.gemstone.FeeUnitType.GWEI -> "${rate.text()} ${context.getString(R.string.fee_rate_gwei)}"
        uniffi.gemstone.FeeUnitType.NATIVE -> rate.text()
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
    GemEmptyStateText.VALIDATORS_TITLE -> context.getString(R.string.stake_state_empty_validators_title)
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
    GemErrorText.AuthenticationUnavailable -> context.getString(R.string.errors_authentication_unavailable)
    GemErrorText.AuthenticationLockedOut -> context.getString(R.string.errors_authentication_locked_out)
    GemErrorText.AuthenticationFailed -> context.getString(R.string.errors_authentication_failed)
    GemErrorText.ConnectionExpired -> context.getString(R.string.errors_connections_expired)
    GemErrorText.ConnectionNotFound -> context.getString(R.string.errors_connections_not_found)
    GemErrorText.RelayUnavailable -> context.getString(R.string.errors_connections_relay_unavailable)
    GemErrorText.Unknown -> context.getString(R.string.errors_unknown)
    is GemErrorText.Message -> text
}

fun PaymentStatus.errorText(context: Context): String = context.getString(R.string.errors_payment_status, context.getString(stringRes()))

@StringRes
private fun PaymentStatus.stringRes(): Int = when (this) {
    PaymentStatus.REQUIRES_ACTION, PaymentStatus.FAILED -> R.string.transaction_status_failed
    PaymentStatus.PROCESSING -> R.string.transaction_status_inprogress
    PaymentStatus.SUCCEEDED -> R.string.transaction_status_completed
    PaymentStatus.EXPIRED -> R.string.transaction_status_expired
    PaymentStatus.CANCELLED -> R.string.errors_cancelled
}

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
    GemListSectionTitle.INFO -> R.string.common_info
    GemListSectionTitle.COMMUNITY -> R.string.settings_community
    GemListSectionTitle.MANAGE -> R.string.common_manage
    GemListSectionTitle.RESOURCES -> R.string.asset_resources
    GemListSectionTitle.SOCIAL_LINKS -> R.string.social_links
    GemListSectionTitle.PROPERTIES -> R.string.nft_properties
}

fun GemListRowTitle.text(context: Context): String = when (this) {
    GemListRowTitle.API -> "API"
    GemListRowTitle.STREAM -> context.getString(R.string.nodes_stream)
    GemListRowTitle.GEM_WALLET_NODE -> context.getString(R.string.nodes_gem_wallet_node)
    GemListRowTitle.NAME -> context.getString(R.string.asset_name)
    GemListRowTitle.NETWORK -> context.getString(R.string.transfer_network)
    GemListRowTitle.ADDRESS -> context.getString(R.string.common_address)
    GemListRowTitle.AVAILABLE -> context.getString(R.string.asset_balances_available)
    GemListRowTitle.STAKE -> context.getString(R.string.wallet_stake)
    GemListRowTitle.FREEZE -> context.getString(R.string.transfer_freeze_title)
    GemListRowTitle.UNFREEZE -> context.getString(R.string.transfer_unfreeze_title)
    GemListRowTitle.CLAIM_REWARDS -> context.getString(R.string.transfer_claim_rewards_title)
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
    GemListRowTitle.SUSPICIOUS_ADDRESS -> context.getString(R.string.common_suspicious_address)
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
    GemListRowTitle.ESTIMATED_CONFIRMATION -> context.getString(R.string.transaction_estimated_confirmation)
    GemListRowTitle.MARKET_PRICE -> context.getString(R.string.perpetual_market_price)
    GemListRowTitle.RATE -> context.getString(R.string.buy_rate)
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
    GemListRowTitle.APP -> context.getString(R.string.wallet_connect_app)
    GemListRowTitle.MEMO -> context.getString(R.string.transfer_memo)
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

fun GemSlippageFooter.text(context: Context): String = when (this) {
    is GemSlippageFooter.Minimum -> context.getString(R.string.common_minimum_value, value.text())
    is GemSlippageFooter.Maximum -> context.getString(R.string.common_maximum_value, value.text())
    GemSlippageFooter.Warning -> context.getString(R.string.swap_slippage_warning)
}

@StringRes
fun GemTriggerOrder.stringRes(): Int = when (this) {
    GemTriggerOrder.TAKE_PROFIT -> R.string.perpetual_take_profit
    GemTriggerOrder.STOP_LOSS -> R.string.perpetual_stop_loss
}

@StringRes
fun AddressType.stringRes(): Int = when (this) {
    AddressType.ADDRESS -> R.string.common_address
    AddressType.CONTRACT -> R.string.asset_contract
    AddressType.ASSET -> R.string.common_token
    AddressType.VALIDATOR -> R.string.stake_validator
    AddressType.CONTACT -> R.string.contacts_contact
    AddressType.INTERNAL_WALLET -> R.string.common_wallet
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

@StringRes
private fun GemPriceAlertLabel.stringRes(): Int = when (this) {
    GemPriceAlertLabel.OVER -> R.string.price_alerts_direction_over
    GemPriceAlertLabel.UNDER -> R.string.price_alerts_direction_under
    GemPriceAlertLabel.INCREASES_BY -> R.string.price_alerts_direction_increases_by
    GemPriceAlertLabel.DECREASES_BY -> R.string.price_alerts_direction_decreases_by
}

fun GemInfoTitle.string(context: Context): String = when (this) {
    GemInfoTitle.NetworkFee -> context.getString(R.string.info_network_fee_title)

    is GemInfoTitle.BalanceRequired -> context.getString(R.string.info_balance_required_title, symbol)

    is GemInfoTitle.TransactionState -> context.getString(state.toPrimitives().statusLabelRes())

    GemInfoTitle.EstimatedConfirmation -> context.getString(R.string.transaction_estimated_confirmation)

    GemInfoTitle.WatchWallet -> context.getString(R.string.info_watch_wallet_title)

    GemInfoTitle.PaymentVerification -> context.getString(R.string.info_payment_verification_title)

    GemInfoTitle.LockTime -> context.getString(R.string.stake_lock_time)

    GemInfoTitle.Apr -> context.getString(R.string.stake_apr, "")

    GemInfoTitle.PriceImpact -> context.getString(R.string.swap_price_impact)

    GemInfoTitle.Slippage -> context.getString(R.string.swap_slippage)

    GemInfoTitle.NoQuote -> context.getString(R.string.errors_swap_no_quote_available)

    is GemInfoTitle.AssetStatus -> when (status) {
        uniffi.gemstone.VerificationStatus.VERIFIED -> ""
        uniffi.gemstone.VerificationStatus.UNVERIFIED -> context.getString(R.string.asset_verification_unverified)
        uniffi.gemstone.VerificationStatus.SUSPICIOUS -> context.getString(R.string.asset_verification_suspicious)
    }

    GemInfoTitle.AccountMinimumBalance -> context.getString(R.string.info_account_minimum_balance_title)

    GemInfoTitle.MinimumAmount -> context.getString(R.string.info_minimum_amount_title)

    GemInfoTitle.StakingReservedFees -> context.getString(R.string.info_stake_reserved_title)

    GemInfoTitle.Pending -> context.getString(R.string.stake_pending)

    GemInfoTitle.StakeFrozenRequired -> context.getString(R.string.info_stake_frozen_required_title)

    GemInfoTitle.FundingApr -> context.getString(R.string.info_perpetual_funding_apr_title)

    GemInfoTitle.FundingPayments -> context.getString(R.string.info_perpetual_funding_payments_title)

    GemInfoTitle.LiquidationPrice -> context.getString(R.string.info_perpetual_liquidation_price_title)

    GemInfoTitle.OpenInterest -> context.getString(R.string.info_perpetual_open_interest_title)

    GemInfoTitle.AutoClose -> context.getString(R.string.perpetual_auto_close)

    GemInfoTitle.MaliciousTransaction -> context.getString(R.string.errors_scan_transaction_malicious_title)

    GemInfoTitle.Warning -> context.getString(R.string.common_warning)

    GemInfoTitle.TransferError -> context.getString(R.string.errors_transfer_error)

    GemInfoTitle.FullyDilutedValuation -> context.getString(R.string.info_fully_diluted_valuation_title)

    GemInfoTitle.CirculatingSupply -> context.getString(R.string.asset_circulating_supply)

    GemInfoTitle.TotalSupply -> context.getString(R.string.asset_total_supply)

    GemInfoTitle.MaxSupply -> context.getString(R.string.info_max_supply_title)

    is GemInfoTitle.WalletName -> name
}

fun GemInfoDescription.string(context: Context): String = when (this) {
    is GemInfoDescription.NetworkFee -> context.getString(R.string.info_network_fee_description, network.bold(), symbol.bold())

    is GemInfoDescription.BalanceRequired -> context.getString(R.string.info_balance_required_description, required.bold(), available.bold(), shortfall.bold())

    is GemInfoDescription.InsufficientNetworkFeeBalance -> context.getString(
        R.string.info_insufficient_network_fee_balance_description,
        required.bold(),
        network.bold(),
        available.text().bold(),
        shortfall.bold(),
    )

    is GemInfoDescription.InsufficientNetworkFee -> context.getString(R.string.transfer_insufficient_network_fee_balance, title.bold())

    is GemInfoDescription.TransactionState -> context.getString(tone.infoDescriptionRes())

    is GemInfoDescription.EstimatedConfirmation -> context.getString(R.string.info_estimated_confirmation_description, network.bold())

    GemInfoDescription.WatchWallet -> context.getString(R.string.info_watch_wallet_description)

    GemInfoDescription.PaymentVerification -> context.getString(R.string.info_payment_verification_description)

    GemInfoDescription.LockTime -> context.getString(R.string.info_lock_time_description)

    GemInfoDescription.Apr -> context.getString(R.string.info_stake_apr_description)

    GemInfoDescription.PriceImpact -> context.getString(R.string.info_price_impact_description)

    GemInfoDescription.Slippage -> context.getString(R.string.info_slippage_description)

    GemInfoDescription.NoQuote -> context.getString(R.string.info_no_quote_description)

    is GemInfoDescription.AssetStatus -> when (status) {
        uniffi.gemstone.VerificationStatus.VERIFIED -> ""
        uniffi.gemstone.VerificationStatus.UNVERIFIED -> context.getString(R.string.info_asset_status_unverified_description)
        uniffi.gemstone.VerificationStatus.SUSPICIOUS -> context.getString(R.string.info_asset_status_suspicious_description)
    }

    is GemInfoDescription.AccountMinimumBalance -> context.getString(R.string.transfer_minimum_account_balance, amount.bold())

    is GemInfoDescription.MinimumAmount -> context.getString(R.string.info_minimum_amount_description, network.bold(), amount.text().bold())

    is GemInfoDescription.SwapMinimumAmount -> context.getString(
        R.string.info_swap_minimum_amount_description,
        provider.bold(),
        required.bold(),
        available.bold(),
        shortfall.bold(),
    )

    GemInfoDescription.StakingReservedFees -> context.getString(R.string.info_stake_reserved_description)

    GemInfoDescription.Pending -> context.getString(R.string.info_transaction_pending_description)

    GemInfoDescription.StakeFrozenRequired -> context.getString(R.string.info_stake_frozen_required_description)

    GemInfoDescription.FundingApr -> context.getString(R.string.info_perpetual_funding_apr_description)

    GemInfoDescription.FundingPayments -> context.getString(R.string.info_perpetual_funding_payments_description)

    GemInfoDescription.LiquidationPrice -> context.getString(R.string.info_perpetual_liquidation_price_description)

    GemInfoDescription.OpenInterest -> context.getString(R.string.info_perpetual_open_interest_description)

    GemInfoDescription.AutoClose -> context.getString(R.string.info_perpetual_auto_close_description)

    GemInfoDescription.MaliciousTransaction -> context.getString(R.string.errors_scan_transaction_malicious_description)

    is GemInfoDescription.MemoRequired -> context.getString(R.string.errors_scan_transaction_memo_required, symbol.bold())

    is GemInfoDescription.DustThreshold -> context.getString(R.string.errors_dust_threshold, network.bold())

    GemInfoDescription.FullyDilutedValuation -> context.getString(R.string.info_fully_diluted_valuation_description)

    GemInfoDescription.CirculatingSupply -> context.getString(R.string.info_circulating_supply_description)

    GemInfoDescription.TotalSupply -> context.getString(R.string.info_total_supply_description)

    GemInfoDescription.MaxSupply -> context.getString(R.string.info_max_supply_description)

    GemInfoDescription.ExistingWalletImported -> context.getString(R.string.wallet_import_already_imported_message)
}

fun GemInfoAmount.text(): String = fiat?.let { "${amount.text()} (~${it.text()})" } ?: amount.text()

fun GemInfoAction.label(context: Context): String = when (this) {
    is GemInfoAction.LearnMore -> context.getString(R.string.common_learn_more)
    is GemInfoAction.Buy -> context.getString(R.string.asset_buy_asset, symbol)
    is GemInfoAction.Acquire -> acquire.flow.actionLabel(context, asset.symbol)
    GemInfoAction.Continue -> context.getString(R.string.common_continue)
}

fun GemAcquireAssetFlow.actionLabel(context: Context, symbol: String): String = context.getString(
    when (this) {
        GemAcquireAssetFlow.OPTIONS -> R.string.asset_get_asset
        GemAcquireAssetFlow.FIAT -> R.string.asset_buy_asset
    },
    symbol,
)

private fun String.bold(): String = "**$this**"

private fun GemFormattedNumber?.bold(): String = this?.text()?.bold().orEmpty()

private fun GemInfoAmount?.bold(): String = this?.text()?.bold().orEmpty()

@StringRes
fun GemAcceptTermsItem.stringRes(): Int = when (this) {
    GemAcceptTermsItem.SELF_CUSTODY -> R.string.onboarding_accept_terms_item1_message
    GemAcceptTermsItem.RECOVERY -> R.string.onboarding_accept_terms_item2_message
    GemAcceptTermsItem.RESPONSIBILITY -> R.string.onboarding_accept_terms_item3_message
}

@StringRes
fun ConnectionStatus.stringRes(): Int? = when (this) {
    ConnectionStatus.Online -> null
    ConnectionStatus.NoInternet -> R.string.errors_no_internet_connection
    ConnectionStatus.NoService -> R.string.errors_no_service_connection
}

@StringRes
fun WalletSource.stringRes(): Int = when (this) {
    WalletSource.Create -> R.string.wallet_new_title
    WalletSource.Import -> R.string.wallet_import_title
}

@StringRes
fun GemTransactionParticipantRole.stringRes(): Int = when (this) {
    GemTransactionParticipantRole.RECIPIENT -> R.string.transaction_recipient
    GemTransactionParticipantRole.SENDER -> R.string.transaction_sender
    GemTransactionParticipantRole.CONTRACT -> R.string.asset_contract
    GemTransactionParticipantRole.VALIDATOR -> R.string.stake_validator
    GemTransactionParticipantRole.PROVIDER -> R.string.common_provider
}

@StringRes
fun GemSwapProgressStep.stringRes(): Int? = when (this) {
    GemSwapProgressStep.COMPLETED -> R.string.transaction_status_completed
    GemSwapProgressStep.PENDING -> R.string.transaction_status_inprogress
    GemSwapProgressStep.WAITING -> null
    GemSwapProgressStep.FAILED -> R.string.transaction_status_failed
    GemSwapProgressStep.REVERTED -> R.string.transaction_status_reverted
    GemSwapProgressStep.REFUNDED -> R.string.transaction_status_refunded
}

@StringRes
fun PortfolioChartType.stringRes(): Int = when (this) {
    PortfolioChartType.VALUE -> R.string.perpetual_value
    PortfolioChartType.PNL -> R.string.perpetual_pnl
}

@StringRes
fun PortfolioType.stringRes(): Int = when (this) {
    PortfolioType.Wallet -> R.string.wallet_portfolio_title
    PortfolioType.Perpetuals -> R.string.perpetuals_title
}

@StringRes
fun GemPriceAlertToggle.toastRes(): Int = when (this) {
    GemPriceAlertToggle.ENABLED -> R.string.price_alerts_disabled_for
    GemPriceAlertToggle.DISABLED -> R.string.price_alerts_enabled_for
}

@StringRes
fun GemSelectAssetTitle.stringRes(): Int = when (this) {
    GemSelectAssetTitle.SEND -> R.string.wallet_send
    GemSelectAssetTitle.PAY_WITH -> R.string.transfer_pay_with
    GemSelectAssetTitle.RECEIVE -> R.string.wallet_receive
    GemSelectAssetTitle.RECEIVE_COLLECTION -> R.string.wallet_receive_collection
    GemSelectAssetTitle.BUY -> R.string.wallet_buy
    GemSelectAssetTitle.SWAP_PAY -> R.string.swap_you_pay
    GemSelectAssetTitle.SWAP_RECEIVE -> R.string.swap_you_receive
    GemSelectAssetTitle.MANAGE_TOKEN_LIST -> R.string.wallet_manage_token_list
    GemSelectAssetTitle.SELECT_ASSET -> R.string.assets_select_asset
    GemSelectAssetTitle.DEPOSIT -> R.string.wallet_deposit
    GemSelectAssetTitle.WITHDRAW -> R.string.wallet_withdraw
    GemSelectAssetTitle.SEARCH -> R.string.assets_select_asset
}

fun GemWalletConnectFailure.text(context: Context): String = when (this) {
    GemWalletConnectFailure.MaliciousOrigin -> context.getString(R.string.errors_connections_malicious_origin)
    GemWalletConnectFailure.Expired -> context.getString(R.string.wallet_connect_request_expired)
    is GemWalletConnectFailure.Failed -> error.text(context)
}

@StringRes
fun FiatQuoteType.titleRes(): Int = when (this) {
    FiatQuoteType.Buy -> R.string.buy_title
    FiatQuoteType.Sell -> R.string.sell_title
}

@StringRes
fun FiatQuoteType.actionRes(): Int = when (this) {
    FiatQuoteType.Buy -> R.string.wallet_buy
    FiatQuoteType.Sell -> R.string.wallet_sell
}

@StringRes
fun GemFiatButtonAction.stringRes(): Int = when (this) {
    GemFiatButtonAction.CONTINUE -> R.string.common_continue
    GemFiatButtonAction.RETRY_QUOTE -> R.string.common_try_again
}

fun GemFiatAmountError.string(context: Context): String = when (this) {
    GemFiatAmountError.InvalidAmount -> context.getString(R.string.errors_invalid_amount)
    is GemFiatAmountError.BelowMinimum -> context.getString(R.string.transfer_minimum_amount, minimum.text())
    is GemFiatAmountError.AboveMaximum -> context.getString(R.string.transfer_maximum_amount, maximum.text())
    is GemFiatAmountError.InsufficientBalance -> context.getString(R.string.transfer_insufficient_balance, title)
}

fun GemFiatViewState.quotesMessage(context: Context): String? = when (val message = quotesMessage()) {
    GemFiatQuotesMessage.EnterAmount -> context.getString(
        R.string.input_enter_amount_to,
        context.getString(quoteType.toPrimitives().actionRes()),
    )

    GemFiatQuotesMessage.NoResults -> context.getString(R.string.buy_no_results)

    is GemFiatQuotesMessage.Failed -> message.error.text(context)

    null -> null
}

@Composable
fun GemConfirmTitle.string(): String = when (this) {
    GemConfirmTitle.Send -> stringResource(R.string.transfer_send_title)
    GemConfirmTitle.Deposit -> stringResource(R.string.wallet_deposit)
    GemConfirmTitle.Withdraw -> stringResource(R.string.transfer_withdraw_title)
    GemConfirmTitle.Swap -> stringResource(R.string.wallet_swap)
    GemConfirmTitle.Approve -> stringResource(R.string.transfer_approve_title)
    GemConfirmTitle.Request -> stringResource(R.string.transfer_review_request)
    GemConfirmTitle.Payment -> stringResource(R.string.transfer_payment_title)
    GemConfirmTitle.Stake -> stringResource(R.string.transfer_stake_title)
    GemConfirmTitle.Unstake -> stringResource(R.string.transfer_unstake_title)
    GemConfirmTitle.Redelegate -> stringResource(R.string.transfer_redelegate_title)
    GemConfirmTitle.ClaimRewards -> stringResource(R.string.transfer_claim_rewards_title)
    GemConfirmTitle.Freeze -> stringResource(R.string.transfer_freeze_title)
    GemConfirmTitle.Unfreeze -> stringResource(R.string.transfer_unfreeze_title)
    GemConfirmTitle.ActivateAsset -> stringResource(R.string.transfer_activate_asset_title)
    is GemConfirmTitle.PerpetualOpen -> direction.toPrimitives().title()
    is GemConfirmTitle.PerpetualIncrease -> stringResource(R.string.perpetual_increase_direction, direction.toPrimitives().title())
    is GemConfirmTitle.PerpetualReduce -> stringResource(R.string.perpetual_reduce_direction, direction.toPrimitives().title())
    GemConfirmTitle.PerpetualClose -> stringResource(R.string.perpetual_close_position)
    GemConfirmTitle.PerpetualModify -> stringResource(R.string.perpetual_modify_position)
}

@Composable
fun FeeUnitType.suffix(assetSymbol: String): String = when (this) {
    FeeUnitType.SatVb -> stringResource(R.string.fee_rate_satvB)
    FeeUnitType.Gwei -> stringResource(R.string.fee_rate_gwei)
    FeeUnitType.Native -> assetSymbol
}

fun GemConfirmErrorDisplay.text(context: Context): String = when (this) {
    is GemConfirmErrorDisplay.Offline -> context.getString(R.string.errors_network_offline)

    is GemConfirmErrorDisplay.Malicious -> context.getString(R.string.errors_scan_transaction_malicious_description)

    is GemConfirmErrorDisplay.MemoRequired -> context.getString(R.string.errors_scan_transaction_memo_required, symbol)

    is GemConfirmErrorDisplay.FeeRatesMissing -> context.getString(R.string.errors_unable_estimate_network_fee)

    is GemConfirmErrorDisplay.Cancelled -> context.getString(R.string.errors_cancelled)

    is GemConfirmErrorDisplay.AccountMissing -> context.getString(R.string.errors_wallet_account_missing)

    is GemConfirmErrorDisplay.Unknown -> context.getString(R.string.errors_unknown)

    is GemConfirmErrorDisplay.BalanceRequired -> context.getString(
        R.string.info_balance_required_description,
        requirement.required.text().boldMarkdown(),
        requirement.available.text().boldMarkdown(),
        requirement.shortfall.text().boldMarkdown(),
    )

    is GemConfirmErrorDisplay.NetworkFeeRequired -> context.getString(
        R.string.info_insufficient_network_fee_balance_description,
        requirement.required.text().boldMarkdown(),
        asset.toPrimitives().id.chain.networkName().boldMarkdown(),
        requirement.available.text().boldMarkdown(),
        requirement.shortfall.text().boldMarkdown(),
    )

    is GemConfirmErrorDisplay.NetworkFeeMissing ->
        context.getString(R.string.transfer_insufficient_network_fee_balance, title.boldMarkdown())

    is GemConfirmErrorDisplay.MinimumAccountBalance ->
        context.getString(R.string.transfer_minimum_account_balance, required.text().boldMarkdown())

    is GemConfirmErrorDisplay.DestinationAccountActivation ->
        context.getString(R.string.transfer_destination_account_activation, required.text().boldMarkdown())

    is GemConfirmErrorDisplay.SwapMinimum -> context.getString(
        R.string.info_swap_minimum_amount_description,
        providerName.boldMarkdown(),
        requirement.required.text().boldMarkdown(),
        requirement.available.text().boldMarkdown(),
        requirement.shortfall.text().boldMarkdown(),
    )

    is GemConfirmErrorDisplay.DustThreshold -> context.getString(R.string.errors_dust_threshold_short)

    is GemConfirmErrorDisplay.InsufficientFunds -> context.getString(R.string.info_insufficient_balance_title)

    is GemConfirmErrorDisplay.Payment -> status.errorText(context)

    is GemConfirmErrorDisplay.Message -> msg
}

@StringRes
fun GemConfirmDestination.title(): Int = when (this) {
    is GemConfirmDestination.Recipient -> R.string.transfer_recipient_title
    is GemConfirmDestination.Contract -> R.string.asset_contract
    is GemConfirmDestination.Validator -> R.string.stake_validator
    is GemConfirmDestination.Resource -> R.string.stake_resource
    is GemConfirmDestination.Provider -> R.string.common_provider
}

fun GemConfirmButtonKind.label(context: Context): String = when (this) {
    GemConfirmButtonKind.CONFIRM -> context.getString(R.string.transfer_confirm)
    GemConfirmButtonKind.RETRY -> context.getString(R.string.common_try_again)
    GemConfirmButtonKind.ACCOUNT_MISSING -> context.getString(R.string.errors_wallet_account_missing)
}

fun Throwable.broadcastLabel(context: Context): String = (this as? GemConfirmException)?.display()?.text(context)
    ?: errorTextOrNull()?.text(context)
    ?: "${context.getString(R.string.errors_transfer_error)}: ${message ?: toString()}"

fun GemSubmitMessage.text(context: Context): String = when (this) {
    is GemSubmitMessage.Warning -> text.text(context)
    is GemSubmitMessage.Confirmed -> text.string(context)
}

@StringRes
fun GemSecurityReminderItem.titleRes(): Int = when (this) {
    GemSecurityReminderItem.KEEP_SAFE -> R.string.onboarding_security_create_wallet_keep_safe_title
    GemSecurityReminderItem.DO_NOT_SHARE -> R.string.onboarding_security_create_wallet_do_not_share_title
    GemSecurityReminderItem.NO_RECOVERY -> R.string.onboarding_security_create_wallet_no_recovery_title
}

@StringRes
fun GemSecurityReminderItem.subtitleRes(): Int = when (this) {
    GemSecurityReminderItem.KEEP_SAFE -> R.string.onboarding_security_create_wallet_keep_safe_subtitle
    GemSecurityReminderItem.DO_NOT_SHARE -> R.string.onboarding_security_create_wallet_do_not_share_subtitle
    GemSecurityReminderItem.NO_RECOVERY -> R.string.onboarding_security_create_wallet_no_recovery_subtitle
}

@StringRes
fun GemDelegationAction.stringRes(): Int = when (this) {
    GemDelegationAction.REDELEGATE -> R.string.transfer_redelegate_title
    GemDelegationAction.STAKE -> R.string.transfer_stake_title
    GemDelegationAction.UNSTAKE -> R.string.transfer_unstake_title
    GemDelegationAction.WITHDRAW -> R.string.transfer_withdraw_title
    GemDelegationAction.DEPOSIT -> R.string.wallet_deposit
}

@StringRes
fun GemStakeSection.stringRes(): Int = when (this) {
    GemStakeSection.MANAGE -> R.string.common_manage
    GemStakeSection.RESOURCES -> R.string.asset_resources
    GemStakeSection.DELEGATIONS -> R.string.stake_delegations
}

@StringRes
fun GemWalletImportKind.tabStringRes(): Int = when (this) {
    GemWalletImportKind.ADDRESS -> R.string.common_address
    GemWalletImportKind.PHRASE -> R.string.common_phrase
    GemWalletImportKind.PRIVATE_KEY -> R.string.common_private_key
}

@StringRes
fun GemWalletImportKind.fieldStringRes(): Int = when (this) {
    GemWalletImportKind.ADDRESS -> R.string.wallet_import_address_field
    GemWalletImportKind.PHRASE -> R.string.common_secret_phrase
    GemWalletImportKind.PRIVATE_KEY -> R.string.common_private_key
}

@StringRes
fun ReportReason.stringRes(): Int = when (this) {
    ReportReason.Spam -> R.string.nft_report_reason_spam
    ReportReason.Malicious -> R.string.nft_report_reason_malicious
    ReportReason.Inappropriate -> R.string.nft_report_reason_inappropriate
    ReportReason.Copyright -> R.string.nft_report_reason_copyright
    ReportReason.Other -> R.string.nft_report_reason_other
}

@StringRes
fun GemCollectibleAction.stringRes(): Int = when (this) {
    GemCollectibleAction.SAVE_IMAGE -> R.string.nft_save_to_photos
    GemCollectibleAction.SET_AVATAR -> R.string.nft_set_as_avatar
    GemCollectibleAction.REFRESH -> R.string.common_refresh
    GemCollectibleAction.REPORT -> R.string.nft_report_report_button_title
}

@StringRes
fun GemNftList.stringRes(): Int = when (this) {
    GemNftList.COLLECTIONS,
    GemNftList.COLLECTION,
    GemNftList.AVATAR,
    -> R.string.nft_collections

    GemNftList.UNVERIFIED -> R.string.asset_verification_unverified
}

@StringRes
fun GemPerpetualSection.stringRes(): Int = when (this) {
    is GemPerpetualSection.Position -> R.string.perpetual_position
    is GemPerpetualSection.Info -> R.string.common_info
}

@StringRes
fun GemPerpetualButton.stringRes(): Int = when (this) {
    GemPerpetualButton.LONG -> R.string.perpetual_long
    GemPerpetualButton.SHORT -> R.string.perpetual_short
    GemPerpetualButton.MODIFY -> R.string.perpetual_modify
    GemPerpetualButton.CLOSE -> R.string.perpetual_close_position
    GemPerpetualButton.INCREASE -> R.string.perpetual_increase_position
    GemPerpetualButton.REDUCE -> R.string.perpetual_reduce_position
}

@StringRes
fun GemPerpetualMarketSection.stringRes(): Int? = when (this) {
    GemPerpetualMarketSection.POSITIONS -> R.string.perpetual_positions
    GemPerpetualMarketSection.PINNED -> R.string.common_pinned
    GemPerpetualMarketSection.MARKETS -> R.string.perpetuals_markets
    GemPerpetualMarketSection.RECENTS, GemPerpetualMarketSection.EMPTY -> null
}

@StringRes
fun GemPerpetualChartLineKind.stringRes(): Int = when (this) {
    GemPerpetualChartLineKind.ENTRY -> R.string.charts_entry
    GemPerpetualChartLineKind.LIQUIDATION -> R.string.perpetual_liquidation
    GemPerpetualChartLineKind.STOP_LOSS -> R.string.perpetual_stop_loss
    GemPerpetualChartLineKind.TAKE_PROFIT -> R.string.perpetual_take_profit
}

fun GemReceiveWarning.text(context: Context): String = when (this) {
    is GemReceiveWarning.AssetNetwork -> context.getString(
        R.string.receive_warning,
        symbol.boldMarkdown(),
        network.boldMarkdown(),
    )

    GemReceiveWarning.NoDestinationTagRequired -> context.getString(R.string.wallet_receive_no_destination_tag_required)

    GemReceiveWarning.NoMemoRequired -> context.getString(R.string.wallet_receive_no_memo_required)
}

@StringRes
fun GemNodeCheckRow.stringRes(): Int = when (this) {
    is GemNodeCheckRow.ChainId -> R.string.nodes_import_node_chain_id
    is GemNodeCheckRow.InSync -> R.string.nodes_import_node_in_sync
    is GemNodeCheckRow.LatestBlock -> R.string.nodes_import_node_latest_block
    is GemNodeCheckRow.Latency -> R.string.nodes_import_node_latency
}

fun GemNodeCheckRow.text(context: Context): String = when (this) {
    is GemNodeCheckRow.ChainId -> value
    is GemNodeCheckRow.LatestBlock -> value.text()
    is GemNodeCheckRow.Latency -> context.getString(R.string.common_latency_in_ms, milliseconds.toInt())
    is GemNodeCheckRow.InSync -> ""
}

fun GemNodeRowTitle.string(context: Context): String = text(context.getString(R.string.nodes_gem_wallet_node))

fun GemNodeSubtitle.text(context: Context): String = when (this) {
    is GemNodeSubtitle.LatestBlock -> text(context.getString(R.string.nodes_import_node_latest_block), value?.text())
}

@StringRes
fun GemPriceAlertPrompt.stringRes(): Int = when (this) {
    GemPriceAlertPrompt.TARGET_PRICE -> R.string.price_alerts_set_alert_set_target_price
    GemPriceAlertPrompt.PRICE_OVER -> R.string.price_alerts_set_alert_price_over
    GemPriceAlertPrompt.PRICE_UNDER -> R.string.price_alerts_set_alert_price_under
    GemPriceAlertPrompt.INCREASES_BY -> R.string.price_alerts_set_alert_price_increases_by
    GemPriceAlertPrompt.DECREASES_BY -> R.string.price_alerts_set_alert_price_decreases_by
}

fun GemPriceAlertSectionKind.title(): String? = when (this) {
    GemPriceAlertSectionKind.Auto -> null
    is GemPriceAlertSectionKind.Asset -> name
}

fun GemPriceAlertSectionKind.footer(context: Context): String? = when (this) {
    GemPriceAlertSectionKind.Auto -> context.getString(R.string.price_alerts_auto_footer)
    is GemPriceAlertSectionKind.Asset -> null
}

@StringRes
fun GemLockPeriod.stringRes(): Int = when (this) {
    GemLockPeriod.IMMEDIATE -> R.string.lock_immediately
    GemLockPeriod.ONE_MINUTE -> R.string.lock_one_minute
    GemLockPeriod.FIVE_MINUTES -> R.string.lock_five_minutes
    GemLockPeriod.FIFTEEN_MINUTES -> R.string.lock_fifteen_minutes
    GemLockPeriod.ONE_HOUR -> R.string.lock_one_hour
    GemLockPeriod.SIX_HOURS -> R.string.lock_six_hours
}

@StringRes
fun Appearance.stringRes(): Int = when (this) {
    Appearance.System -> R.string.settings_appearance_system
    Appearance.Light -> R.string.settings_appearance_light
    Appearance.Dark -> R.string.settings_appearance_dark
}

@StringRes
fun GemSwapButtonAction.stringRes(): Int = when (this) {
    GemSwapButtonAction.InsufficientBalance -> R.string.transfer_insufficient_balance

    is GemSwapButtonAction.UseMinimumAmount -> R.string.swap_use_minimum_amount

    GemSwapButtonAction.RetryQuote,
    GemSwapButtonAction.RetryTransfer,
    -> R.string.common_try_again

    GemSwapButtonAction.Swap -> R.string.wallet_swap
}

fun GemSwapErrorDisplay.text(context: Context): String = when (this) {
    is GemSwapErrorDisplay.NotSupportedAsset -> context.getString(R.string.errors_swap_not_supported_asset)
    is GemSwapErrorDisplay.NoQuote -> context.getString(R.string.errors_swap_no_quote_available)
    is GemSwapErrorDisplay.Offline -> context.getString(R.string.errors_network_offline)
    is GemSwapErrorDisplay.MinimumAmount -> context.getString(R.string.errors_swap_minimum_amount, minimum.text().boldMarkdown())
    is GemSwapErrorDisplay.AmountTooSmall -> context.getString(R.string.errors_swap_amount_too_small)
}

fun GemAmountTitle.text(context: Context): String = when (this) {
    GemAmountTitle.Send -> context.getString(R.string.transfer_send_title)
    GemAmountTitle.Deposit -> context.getString(R.string.wallet_deposit)
    GemAmountTitle.Withdraw -> context.getString(R.string.wallet_withdraw)
    GemAmountTitle.Stake -> context.getString(R.string.transfer_stake_title)
    GemAmountTitle.Unstake -> context.getString(R.string.transfer_unstake_title)
    GemAmountTitle.Redelegate -> context.getString(R.string.transfer_redelegate_title)
    GemAmountTitle.Rewards -> context.getString(R.string.transfer_claim_rewards_title)
    GemAmountTitle.Freeze -> context.getString(R.string.transfer_freeze_title)
    GemAmountTitle.Unfreeze -> context.getString(R.string.transfer_unfreeze_title)
    is GemAmountTitle.PerpetualOpen -> context.getString(direction.toPrimitives().stringRes())
    is GemAmountTitle.PerpetualIncrease -> context.getString(R.string.perpetual_increase_direction, context.getString(direction.toPrimitives().stringRes()))
    is GemAmountTitle.PerpetualReduce -> context.getString(R.string.perpetual_reduce_direction, context.getString(direction.toPrimitives().stringRes()))
}

fun GemAmountErrorDisplay.text(context: Context): String = when (this) {
    is GemAmountErrorDisplay.None -> ""
    is GemAmountErrorDisplay.InvalidAmount -> context.getString(R.string.errors_invalid_amount)
    is GemAmountErrorDisplay.BelowMinimum -> context.getString(R.string.transfer_minimum_amount, minimum.text())
    is GemAmountErrorDisplay.InsufficientBalance -> context.getString(R.string.transfer_insufficient_balance, title)
}

fun GemRowText.string(context: Context): String = text.string(context)

fun GemCustomFeeCheck.errorText(context: Context): String? = when (this) {
    is GemCustomFeeCheck.BelowMinimum -> context.getString(R.string.common_minimum_value, rate.string(context))
    is GemCustomFeeCheck.OverMaximum -> context.getString(R.string.common_maximum_value, rate.string(context))
    GemCustomFeeCheck.Valid -> null
}

fun GemBannerButton.stringRes(): Int = when (this) {
    GemBannerButton.BUY -> R.string.wallet_buy
    GemBannerButton.RECEIVE -> R.string.wallet_receive
}

fun GemChainsFilterSummary.text(context: Context): String = when (this) {
    GemChainsFilterSummary.All -> context.getString(R.string.common_all)
    is GemChainsFilterSummary.Chain -> chain.requireChain().networkName()
    is GemChainsFilterSummary.Count -> count.toString()
}

fun GemTransactionsFilterSummary.text(context: Context): String = when (this) {
    GemTransactionsFilterSummary.All -> context.getString(R.string.common_all)
    is GemTransactionsFilterSummary.Filter -> context.getString(filter.getLabel())
    is GemTransactionsFilterSummary.Count -> count.toString()
}

fun VerificationStatus.labelRes(): Int? = when (this) {
    VerificationStatus.Verified -> null
    VerificationStatus.Unverified -> R.string.asset_verification_unverified
    VerificationStatus.Suspicious -> R.string.asset_verification_suspicious
}

fun GemNameIndicator.contentDescription(): Int? = when (this) {
    GemNameIndicator.ERROR -> R.string.errors_error_occurred
    GemNameIndicator.LOADING, GemNameIndicator.SUCCESS -> null
}
