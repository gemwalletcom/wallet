package com.gemwallet.android.ui.components.screen

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.imeAnimationSource
import androidx.compose.foundation.layout.imeAnimationTarget
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SheetValue
import androidx.compose.material3.SheetState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import com.gemwallet.android.ui.components.dialog.DialogBar
import com.gemwallet.android.ui.components.dialog.DialogBarDismissType
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.alpha20
import com.gemwallet.android.ui.theme.SheetSizing

enum class SheetExpansion {
    Partial,
    Full,
}

@Composable
fun <T : Any> ModalBottomSheet(
    item: T?,
    onDismissRequest: () -> Unit,
    modifier: Modifier = Modifier,
    expansion: SheetExpansion = SheetExpansion.Partial,
    containerColor: Color = MaterialTheme.colorScheme.surface,
    shape: Shape = RoundedCornerShape(topStart = SheetSizing.cornerSize, topEnd = SheetSizing.cornerSize),
    title: (@Composable (T) -> String)? = null,
    dismissType: DialogBarDismissType = DialogBarDismissType.Close,
    dragHandle: (@Composable () -> Unit)? = { Box { Spacer16() } },
    content: @Composable ColumnScope.(T) -> Unit,
) {
    var closingItem by remember { mutableStateOf(item) }
    if (item != null) closingItem = item
    val presentedItem = closingItem ?: return

    ModalBottomSheet(
        isVisible = item != null,
        onDismissRequest = onDismissRequest,
        modifier = modifier,
        expansion = expansion,
        containerColor = containerColor,
        shape = shape,
        title = title?.invoke(presentedItem),
        dismissType = dismissType,
        dragHandle = dragHandle,
    ) {
        content(presentedItem)
    }
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)
@Composable
fun ModalBottomSheet(
    isVisible: Boolean,
    onDismissRequest: () -> Unit,
    modifier: Modifier = Modifier,
    expansion: SheetExpansion = SheetExpansion.Partial,
    containerColor: Color = MaterialTheme.colorScheme.surface,
    shape: Shape = RoundedCornerShape(topStart = SheetSizing.cornerSize, topEnd = SheetSizing.cornerSize),
    title: String? = null,
    dismissType: DialogBarDismissType = DialogBarDismissType.Close,
    dragHandle: (@Composable () -> Unit)? = { Box { Spacer16() } },
    content: @Composable ColumnScope.() -> Unit,
) {
    var isPresented by remember { mutableStateOf(isVisible) }

    LaunchedEffect(isVisible) {
        if (isVisible) {
            isPresented = true
        }
    }

    if (!isPresented) return

    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = expansion == SheetExpansion.Full)
    var ownsKeyboard by remember { mutableStateOf(false) }
    LaunchedEffect(isVisible) {
        if (isVisible) {
            sheetState.show()
        } else {
            sheetState.hide()
            isPresented = false
        }
    }

    androidx.compose.material3.ModalBottomSheet(
        onDismissRequest = onDismissRequest,
        modifier = modifier.onFocusChanged { ownsKeyboard = it.hasFocus },
        sheetState = sheetState,
        scrimColor = MaterialTheme.colorScheme.onSurface.copy(alpha = alpha20),
        shape = shape,
        containerColor = containerColor,
        dragHandle = if (title != null) null else dragHandle,
        contentWindowInsets = {
            when {
                sheetState.targetValue == sheetState.currentValue -> BottomSheetDefaults.windowInsets
                sheetState.targetValue == SheetValue.Hidden -> sheetWindowInsets(WindowInsets.imeAnimationSource)
                ownsKeyboard -> sheetWindowInsets(WindowInsets.imeAnimationTarget)
                else -> sheetWindowInsets(WindowInsets())
            }
        },
        content = { SheetContent(sheetState, onDismissRequest, title, dismissType, content) },
    )
}

@Composable
private fun sheetWindowInsets(keyboard: WindowInsets): WindowInsets =
    WindowInsets.systemBars
        .union(WindowInsets.displayCutout)
        .union(keyboard)
        .only(WindowInsetsSides.Vertical)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ColumnScope.SheetContent(
    sheetState: SheetState,
    onDismissRequest: () -> Unit,
    title: String?,
    dismissType: DialogBarDismissType,
    content: @Composable ColumnScope.() -> Unit,
) {
    val keyboard = LocalSoftwareKeyboardController.current
    val isLeaving = sheetState.currentValue != SheetValue.Hidden && sheetState.targetValue == SheetValue.Hidden
    LaunchedEffect(isLeaving) {
        if (isLeaving) {
            keyboard?.hide()
        }
    }
    if (title != null) {
        DialogBar(onDismissRequest = onDismissRequest, title = title, dismissType = dismissType)
    }
    content()
}
