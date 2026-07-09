package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.PortfolioType

internal fun SavedStateHandle.portfolioType(): PortfolioType =
    get<PortfolioType>(RouteArgument.Type.key) ?: PortfolioType.Wallet
