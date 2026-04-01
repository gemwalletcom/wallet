package com.gemwallet.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner

@Composable
fun ObserveStartedState(
    onChanged: (Boolean) -> Unit,
) {
    val lifecycleOwner = LocalLifecycleOwner.current
    val currentOnChanged by rememberUpdatedState(onChanged)

    DisposableEffect(lifecycleOwner) {
        val lifecycle = lifecycleOwner.lifecycle
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_START -> currentOnChanged(true)
                Lifecycle.Event.ON_STOP -> currentOnChanged(false)
                else -> Unit
            }
        }

        currentOnChanged(lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED))
        lifecycle.addObserver(observer)

        onDispose {
            lifecycle.removeObserver(observer)
            currentOnChanged(false)
        }
    }
}
