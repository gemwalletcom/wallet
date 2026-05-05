package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.referral.views.ReferralNavScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data class ReferralRoute(val code: String? = null) : NavKey

fun EntryProviderScope<NavKey>.referral(
    onClose: () -> Unit,
) {
    entry<ReferralRoute>(
        metadata = { key -> routeArguments(RouteArgument.Code to key.code) },
    ) {
        ReferralNavScreen(onClose)
    }
}
