package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
internal fun AddressCard(row: GemListRowUIModel.Address, onCopy: () -> Unit) {
    Text(
        modifier = Modifier
            .fillMaxWidth()
            .listItem(position = ListPosition.Single)
            .clickable(onCopy)
            .padding(paddingDefault),
        text = row.address,
        textAlign = TextAlign.Center,
        color = MaterialTheme.colorScheme.secondary,
        fontWeight = FontWeight.Medium,
        style = MaterialTheme.typography.bodyMedium,
    )
}
