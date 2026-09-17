package com.gemwallet.android.ui.components.fields

import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.components.image.vector
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.theme.smallIconSize

@Composable
fun NameResolveIndicator(model: NameResolveIndicatorUIModel?) {
    when (model) {
        NameResolveIndicatorUIModel.Loading -> CircularProgressIndicator16()
        is NameResolveIndicatorUIModel.Icon -> Icon(
            modifier = Modifier.size(smallIconSize),
            imageVector = model.symbol.vector(),
            contentDescription = model.contentDescription?.let { stringResource(it) },
            tint = model.style.color(),
        )
        null -> Unit
    }
}
