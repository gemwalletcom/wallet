package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.style.icon
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import uniffi.gemstone.GemAssetSectionKind

@Composable
fun AssetSectionHeaderItem(kind: GemAssetSectionKind) {
    val title = kind.titleRes() ?: return
    val icon = kind.icon() ?: return
    Row(
        modifier = Modifier
            .sectionHeaderItem(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        Icon(
            modifier = Modifier.size(paddingDefault),
            imageVector = icon,
            tint = MaterialTheme.colorScheme.secondary,
            contentDescription = "pinned_section",
        )
        Text(
            modifier = Modifier
                .fillMaxWidth(),
            text = stringResource(title),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.secondary,
        )
    }
}
