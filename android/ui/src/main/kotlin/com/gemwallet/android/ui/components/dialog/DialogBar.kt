package com.gemwallet.android.ui.components.dialog

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.SheetDragHandle
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingSmall

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DialogBar(onDismissRequest: () -> Unit, title: String? = null, dismissType: DialogBarDismissType = DialogBarDismissType.Close, actions: @Composable RowScope.() -> Unit = {}) {
    Box {
        CenterAlignedTopAppBar(
            title = {
                if (title != null) {
                    Text(
                        text = title,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            },
            navigationIcon = {
                if (dismissType == DialogBarDismissType.Close) {
                    DismissButton(dismissType, onDismissRequest)
                }
            },
            actions = {
                actions()
                if (dismissType == DialogBarDismissType.Confirm) {
                    DismissButton(dismissType, onDismissRequest)
                }
            },
            windowInsets = WindowInsets(0),
            colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent),
        )
        if (title != null) {
            SheetDragHandle(modifier = Modifier.align(Alignment.TopCenter))
        }
    }
}

@Composable
private fun DismissButton(dismissType: DialogBarDismissType, onClick: () -> Unit) {
    IconButton(
        modifier = Modifier.padding(horizontal = paddingSmall),
        onClick = onClick,
        colors = IconButtonDefaults.iconButtonColors(
            containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
        ),
    ) {
        Icon(
            imageVector = when (dismissType) {
                DialogBarDismissType.Close -> AppIcons.Close
                DialogBarDismissType.Confirm -> AppIcons.Check
            },
            contentDescription = stringResource(
                when (dismissType) {
                    DialogBarDismissType.Close -> R.string.common_close
                    DialogBarDismissType.Confirm -> R.string.common_done
                },
            ),
        )
    }
}

enum class DialogBarDismissType {
    Close,
    Confirm,
}
