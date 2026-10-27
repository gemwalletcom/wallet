package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.Text
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.theme.paddingDefault

fun LazyListScope.textMessageContent(message: String) {
    item {
        SubheaderItem(R.string.sign_message_message)
        Text(
            modifier = Modifier
                .fillMaxWidth()
                .listItem()
                .padding(paddingDefault),
            text = message,
        )
    }
}
