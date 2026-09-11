package com.gemwallet.android.ui.components.fields

import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator16
import com.gemwallet.android.ui.icons.AppIcons
import uniffi.gemstone.GemNameRecordState
import com.gemwallet.android.ui.theme.smallIconSize
import androidx.compose.foundation.layout.size

@Composable
fun NameResolveIndicator(state: GemNameRecordState) {
    when (state) {
        is GemNameRecordState.Loading -> CircularProgressIndicator16()
        GemNameRecordState.Error -> Icon(
            modifier = Modifier.size(smallIconSize),
            imageVector = AppIcons.Error,
            contentDescription = stringResource(R.string.errors_error_occurred),
            tint = MaterialTheme.colorScheme.error,
        )
        is GemNameRecordState.Complete -> Icon(
            modifier = Modifier.size(smallIconSize),
            imageVector = AppIcons.CheckCircle,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.tertiary,
        )
        GemNameRecordState.None -> Unit
    }
}
