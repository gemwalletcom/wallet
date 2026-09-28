package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
internal fun AddressRow(address: String, listPosition: ListPosition, modifier: Modifier) {
    Text(
        modifier = Modifier
            .fillMaxWidth()
            .listItem(position = listPosition)
            .then(modifier)
            .padding(paddingDefault),
        text = address,
        textAlign = TextAlign.Center,
        style = MaterialTheme.typography.bodyLarge,
    )
}
