package com.gemwallet.android.ui.components

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.open

@Composable
fun DocsInfoButton(url: String) {
    val uriHandler = LocalUriHandler.current
    val context = LocalContext.current
    IconButton(onClick = { uriHandler.open(context, url) }) {
        Icon(imageVector = AppIcons.InfoOutlined, contentDescription = null)
    }
}
