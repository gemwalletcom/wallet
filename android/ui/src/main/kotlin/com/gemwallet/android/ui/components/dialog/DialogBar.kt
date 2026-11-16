package com.gemwallet.android.ui.components.dialog

import androidx.annotation.StringRes
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.SheetDragHandle
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingSmall

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DialogBar(onDismissRequest: () -> Unit, title: String? = null, onBack: (() -> Unit)? = null, onConfirm: (() -> Unit)? = null, isConfirmEnabled: Boolean = true, actions: @Composable RowScope.() -> Unit = {}) {
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
                when (onBack) {
                    null -> DialogBarButton(AppIcons.Close, R.string.common_close, onDismissRequest)
                    else -> DialogBarButton(AppIcons.ArrowBack, R.string.common_back, onBack)
                }
            },
            actions = {
                actions()
                onConfirm?.let { DialogBarButton(AppIcons.Check, R.string.common_done, it, isConfirmEnabled) }
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
private fun DialogBarButton(icon: ImageVector, @StringRes description: Int, onClick: () -> Unit, enabled: Boolean = true) {
    IconButton(
        modifier = Modifier.padding(horizontal = paddingSmall),
        onClick = onClick,
        enabled = enabled,
        colors = IconButtonDefaults.iconButtonColors(
            containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
        ),
    ) {
        Icon(imageVector = icon, contentDescription = stringResource(description))
    }
}
