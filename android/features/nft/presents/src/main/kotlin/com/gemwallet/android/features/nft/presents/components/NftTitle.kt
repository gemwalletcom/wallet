package com.gemwallet.android.features.nft.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Verified
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.ui.theme.tinyIconSize
import com.wallet.core.primitives.VerificationStatus

@Composable
fun NftTitle(
    name: String,
    status: VerificationStatus,
    modifier: Modifier = Modifier,
    iconSize: Dp = tinyIconSize,
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(space4),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            text = name,
            modifier = Modifier.weight(1f, fill = false),
        )
        if (status == VerificationStatus.Verified) {
            Icon(
                imageVector = Icons.Outlined.Verified,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(iconSize),
            )
        }
    }
}
