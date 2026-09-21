package com.gemwallet.android.features.asset.presents.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PortfolioType
import uniffi.gemstone.PortfolioChartType

@StringRes
internal fun PortfolioChartType.stringRes(): Int = when (this) {
    PortfolioChartType.VALUE -> R.string.perpetual_value
    PortfolioChartType.PNL -> R.string.perpetual_pnl
}

@StringRes
internal fun PortfolioType.stringRes(): Int = when (this) {
    PortfolioType.Wallet -> R.string.wallet_portfolio_title
    PortfolioType.Perpetuals -> R.string.perpetuals_title
}
