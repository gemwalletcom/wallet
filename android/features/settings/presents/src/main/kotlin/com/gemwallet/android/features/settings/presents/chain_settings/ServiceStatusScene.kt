@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.presents.chain_settings

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.gemListSections
import com.gemwallet.android.ui.components.screen.PullToRefreshBox
import com.gemwallet.android.ui.components.screen.Scene
import uniffi.gemstone.GemListSection

@Composable
fun ServiceStatusScene(sections: List<GemListSection>, onRefresh: () -> Unit, onCancel: () -> Unit) {
    Scene(
        title = stringResource(R.string.transaction_status),
        onClose = onCancel,
    ) {
        PullToRefreshBox(
            isRefreshing = false,
            onRefresh = onRefresh,
        ) {
            LazyColumn(modifier = Modifier.fillMaxSize()) {
                gemListSections(sections)
            }
        }
    }
}
