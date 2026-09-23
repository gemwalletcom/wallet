package com.gemwallet.android.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.open
import com.gemwallet.android.ui.theme.Spacer8
import uniffi.gemstone.DocsUrl

@Composable
fun RootWarningDialog(onCancel: () -> Unit, onIgnore: () -> Unit) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current

    AlertDialog(
        onDismissRequest = onIgnore,
        title = { Text(text = stringResource(R.string.rootcheck_security_alert)) },
        text = {
            Column {
                Text(text = stringResource(R.string.rootcheck_body))
                Spacer8()
                Text(
                    modifier = Modifier.clickable {
                        uriHandler.open(context, AppUrl.docs(DocsUrl.RootedDevice))
                    },
                    text = stringResource(R.string.common_learn_more),
                    color = MaterialTheme.colorScheme.primary,
                )
            }
        },
        confirmButton = {
            Button(onClick = onCancel) {
                Text(text = stringResource(R.string.rootcheck_exit))
            }
        },
        dismissButton = {
            Button(onClick = onIgnore) {
                Text(text = stringResource(R.string.rootcheck_ignore))
            }
        },
    )
}
