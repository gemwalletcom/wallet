package com.gemwallet.android.features.nft.presents.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_head.AmountHeadAction
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingDefault

@Composable
fun NftHeaderActions(
    canSend: Boolean,
    onSend: () -> Unit,
    onRefresh: () -> Unit,
    onReport: () -> Unit,
) {
    var actionFontSize by remember { mutableStateOf(16.sp) }
    var isMenuExpanded by remember { mutableStateOf(false) }
    val send = stringResource(R.string.wallet_send)
    val more = stringResource(R.string.wallet_more)

    Row(
        modifier = Modifier.width(IntrinsicSize.Min),
        horizontalArrangement = Arrangement.spacedBy(paddingDefault),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        AmountHeadAction(
            modifier = Modifier.weight(1f),
            title = send,
            imageVector = AppIcons.Send,
            contentDescription = send,
            enabled = canSend,
            fontSize = actionFontSize,
            onNextFontSize = {
                if (actionFontSize > it) actionFontSize = it
            },
            onClick = onSend,
        )
        Box(modifier = Modifier.weight(1f)) {
            AmountHeadAction(
                modifier = Modifier.fillMaxWidth(),
                title = more,
                imageVector = AppIcons.MoreVert,
                contentDescription = more,
                fontSize = actionFontSize,
                onNextFontSize = {
                    if (actionFontSize > it) actionFontSize = it
                },
                onClick = { isMenuExpanded = true },
            )
            DropdownMenu(
                expanded = isMenuExpanded,
                onDismissRequest = { isMenuExpanded = false },
            ) {
                DropdownMenuItem(
                    text = { Text(stringResource(R.string.common_refresh)) },
                    leadingIcon = { Icon(AppIcons.Refresh, contentDescription = null) },
                    onClick = {
                        isMenuExpanded = false
                        onRefresh()
                    },
                )
                DropdownMenuItem(
                    text = { Text(stringResource(R.string.nft_report_report_button_title), color = MaterialTheme.colorScheme.error) },
                    leadingIcon = { Icon(AppIcons.Warning, contentDescription = null, tint = MaterialTheme.colorScheme.error) },
                    onClick = {
                        isMenuExpanded = false
                        onReport()
                    },
                )
            }
        }
    }
}
