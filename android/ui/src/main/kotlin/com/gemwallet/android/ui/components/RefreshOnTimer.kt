package com.gemwallet.android.ui.components

import android.os.SystemClock
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.delay

@Composable
fun RefreshOnTimer(intervalMillis: Long, onRefresh: () -> Unit) {
    val lifecycleOwner = LocalLifecycleOwner.current
    val onRefreshCurrent by rememberUpdatedState(onRefresh)
    var refreshedAt by remember { mutableLongStateOf(SystemClock.elapsedRealtime()) }

    LaunchedEffect(lifecycleOwner, intervalMillis) {
        if (intervalMillis <= 0) return@LaunchedEffect
        lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
            while (true) {
                delay((intervalMillis - (SystemClock.elapsedRealtime() - refreshedAt)).coerceAtLeast(0))
                refreshedAt = SystemClock.elapsedRealtime()
                onRefreshCurrent()
            }
        }
    }
}
