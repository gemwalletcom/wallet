package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.components.image.vector
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.localization.contentDescription
import com.gemwallet.android.ui.style.style
import com.gemwallet.android.ui.style.symbol
import com.gemwallet.android.ui.theme.smallIconSize
import uniffi.gemstone.GemNameIndicator

@Composable
fun NameResolveIndicator(indicator: GemNameIndicator?) {
    when (indicator) {
        GemNameIndicator.LOADING -> CircularProgressIndicator16()

        GemNameIndicator.ERROR, GemNameIndicator.SUCCESS -> Icon(
            modifier = Modifier.size(smallIconSize),
            imageVector = indicator.symbol().vector(),
            contentDescription = indicator.contentDescription()?.let { stringResource(it) },
            tint = indicator.style().color(),
        )

        null -> Unit
    }
}
