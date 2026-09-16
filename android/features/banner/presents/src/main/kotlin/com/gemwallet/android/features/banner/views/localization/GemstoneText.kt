package com.gemwallet.android.features.banner.views.localization

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemBannerAmount
import uniffi.gemstone.GemBannerDescription
import uniffi.gemstone.GemBannerTitle
import uniffi.gemstone.GemValueStyle

@Composable
internal fun bannerTitle(title: GemBannerTitle): String = when (title) {
    is GemBannerTitle.Stake -> stringResource(R.string.banner_stake_title, title.assetName)
    GemBannerTitle.AccountActivation -> stringResource(R.string.banner_account_activation_title)
    GemBannerTitle.Warning -> stringResource(R.string.common_warning)
    GemBannerTitle.ActivateAsset -> stringResource(R.string.transfer_activate_asset_title)
    GemBannerTitle.SuspiciousAsset -> stringResource(R.string.banner_asset_status_title)
    GemBannerTitle.Onboarding -> stringResource(R.string.banner_onboarding_title)
    GemBannerTitle.TradePerpetuals -> stringResource(R.string.banner_perpetuals_title)
}

@Composable
internal fun bannerDescription(description: GemBannerDescription): String = when (description) {
    is GemBannerDescription.Stake -> stringResource(R.string.banner_stake_description, description.assetSymbol)
    is GemBannerDescription.AccountActivation -> stringResource(
        R.string.banner_account_activation_description,
        description.networkName,
        formatAmount(description.fee),
    )
    is GemBannerDescription.MultiSignatureBlocked -> stringResource(R.string.warnings_multi_signature_blocked, description.networkName)
    is GemBannerDescription.ActivateAsset -> stringResource(
        R.string.banner_activate_asset_description,
        description.assetSymbol,
        description.networkName,
    )
    GemBannerDescription.SuspiciousAsset -> stringResource(R.string.banner_asset_status_description)
    GemBannerDescription.Onboarding -> stringResource(R.string.banner_onboarding_description)
    GemBannerDescription.TradePerpetuals -> stringResource(R.string.banner_perpetuals_description)
}

private fun formatAmount(amount: GemBannerAmount): String = ValueFormatter(style = GemValueStyle.AUTO)
    .string(amount.value, decimals = amount.decimals, currency = amount.symbol)
