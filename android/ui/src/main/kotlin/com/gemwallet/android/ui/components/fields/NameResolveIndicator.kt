package com.gemwallet.android.ui.components.fields

import com.gemwallet.android.ui.style.indicator
import com.gemwallet.android.ui.style.NameResolveIndicatorStyle
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import uniffi.gemstone.GemNameRecordState
import com.gemwallet.android.ui.theme.smallIconSize
import androidx.compose.foundation.layout.size

@Composable
fun NameResolveIndicator(state: GemNameRecordState) {
    when (val style = state.indicator()) {
        NameResolveIndicatorStyle.Loading -> CircularProgressIndicator16()
        is NameResolveIndicatorStyle.Icon -> Icon(
            modifier = Modifier.size(smallIconSize),
            imageVector = style.vector,
            contentDescription = style.contentDescription,
            tint = style.tint,
        )
        null -> Unit
    }
}
