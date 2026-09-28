package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.screen.SheetExpansion
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
internal fun TextMessageSheet(isVisible: Boolean, message: String, onDismissRequest: () -> Unit) {
    ModalBottomSheet(
        isVisible = isVisible,
        expansion = SheetExpansion.Full,
        onDismissRequest = onDismissRequest,
        title = stringResource(R.string.sign_message_view_full_message),
    ) {
        LazyColumn(
            contentPadding = PaddingValues(paddingDefault),
        ) {
            item {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = message,
                )
            }
        }
    }
}
