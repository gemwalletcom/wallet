package com.gemwallet.android.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.theme.space0

@Stable
class ConnectionBannerState {
    var title by mutableStateOf<String?>(null)
        private set

    var isDismissed by mutableStateOf(false)
        private set

    val isVisible: Boolean get() = title != null && !isDismissed

    fun update(title: String?) {
        if (title != null && this.title == null) {
            isDismissed = false
        }
        this.title = title
    }

    fun dismiss() {
        isDismissed = true
    }
}

val LocalConnectionBannerState = compositionLocalOf { ConnectionBannerState() }

val LocalConnectionBannerHandled = compositionLocalOf { false }

@Composable
fun ConnectionStatusBannerHost(
    modifier: Modifier = Modifier,
    windowInsets: WindowInsets = WindowInsets(space0),
) {
    if (LocalConnectionBannerHandled.current) return
    val state = LocalConnectionBannerState.current
    var displayTitle by remember { mutableStateOf(state.title ?: "") }
    state.title?.let { title ->
        SideEffect { displayTitle = title }
    }
    AnimatedVisibility(
        visible = state.isVisible,
        enter = expandVertically() + fadeIn(),
        exit = shrinkVertically() + fadeOut(),
    ) {
        ConnectionStatusBanner(
            title = displayTitle,
            onDismiss = state::dismiss,
            modifier = modifier,
            windowInsets = windowInsets,
        )
    }
}
