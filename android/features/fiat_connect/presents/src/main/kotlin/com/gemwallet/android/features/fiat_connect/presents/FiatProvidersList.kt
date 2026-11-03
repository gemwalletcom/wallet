package com.gemwallet.android.features.fiat_connect.presents

import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.remember
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.ProvidersBottomSheet
import com.wallet.core.primitives.FiatProviderName
import uniffi.gemstone.GemProviderKind
import uniffi.gemstone.GemProviderRow

@Composable
fun FiatProvidersList(isShow: MutableState<Boolean>, providers: List<GemProviderRow>, onProviderSelect: (FiatProviderName) -> Unit) {
    val openedProviders = remember(isShow.value) { providers.takeIf { isShow.value } }
    ProvidersBottomSheet(
        providers = openedProviders,
        onSelect = { kind ->
            if (kind is GemProviderKind.Fiat) {
                onProviderSelect(kind.provider.toPrimitives())
                isShow.value = false
            }
        },
        onDismiss = { isShow.value = false },
    )
}
