package com.gemwallet.android.ui.components.list_item.property

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.gemwallet.android.ui.models.ListPosition
import kotlinx.coroutines.delay

private const val TICK_MS = 1000L

@Composable
fun PropertyExpiryItem(
    title: String,
    expiresAt: Long,
    listPosition: ListPosition,
) {
    var remaining by remember(expiresAt) { mutableLongStateOf(expiresAt - System.currentTimeMillis()) }

    LaunchedEffect(expiresAt) {
        while (remaining > 0) {
            delay(TICK_MS)
            remaining = expiresAt - System.currentTimeMillis()
        }
    }

    val seconds = (remaining / TICK_MS).coerceAtLeast(0)
    PropertyItem(
        title = title,
        data = "%d:%02d".format(seconds / 60, seconds % 60),
        listPosition = listPosition,
    )
}
