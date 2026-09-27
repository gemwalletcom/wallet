package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.rewards.presents.RewardsScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data class RewardsRoute(val code: String? = null) : NavKey

fun EntryProviderScope<NavKey>.rewards(onClose: () -> Unit) {
    entry<RewardsRoute>(
        metadata = { key -> routeArguments(RouteArgument.Code to key.code) },
    ) {
        RewardsScreen(onClose)
    }
}
