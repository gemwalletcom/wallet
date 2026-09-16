package com.gemwallet.android.features.settings.networks.presents.style

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import com.gemwallet.android.ui.icons.AppIcons
import uniffi.gemstone.GemNodeSyncState

@Composable
internal fun GemNodeSyncState.icon(): ImageVector = when (this) {
    GemNodeSyncState.IN_SYNC -> AppIcons.CheckCircleOutlined
    GemNodeSyncState.OUT_OF_SYNC -> AppIcons.Cancel
}

@Composable
@ReadOnlyComposable
internal fun GemNodeSyncState.tint(): Color = when (this) {
    GemNodeSyncState.IN_SYNC -> MaterialTheme.colorScheme.tertiary
    GemNodeSyncState.OUT_OF_SYNC -> MaterialTheme.colorScheme.error
}
