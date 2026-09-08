package com.gemwallet.android.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.repeatOnLifecycle
import com.gemwallet.android.domains.connection.refreshInterval
import com.gemwallet.android.ui.LocalConnectionStatus
import kotlinx.coroutines.delay
import uniffi.gemstone.GemRefreshKind

@Composable
fun RefreshOnTimer(kind: GemRefreshKind, onRefresh: () -> Unit) {
    val status by LocalConnectionStatus.current.collectAsStateWithLifecycle()
    val lifecycleOwner = LocalLifecycleOwner.current
    val interval = status.refreshInterval(kind).toMillis()

    LaunchedEffect(lifecycleOwner, interval) {
        lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
            while (true) {
                delay(interval)
                onRefresh()
            }
        }
    }
}
