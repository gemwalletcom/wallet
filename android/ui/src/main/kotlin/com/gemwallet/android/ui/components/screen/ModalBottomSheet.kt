package com.gemwallet.android.ui.components.screen

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SheetState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.alpha20
import com.gemwallet.android.ui.theme.sheetCornerSize

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ModalBottomSheet(
    onDismissRequest: () -> Unit,
    modifier: Modifier = Modifier,
    sheetState: SheetState = rememberModalBottomSheetState(),
    containerColor: Color = MaterialTheme.colorScheme.surface,
    shape: Shape = RoundedCornerShape(topStart = sheetCornerSize, topEnd = sheetCornerSize),
    dragHandle: @Composable () -> Unit = { Box { Spacer16() } },
    content: @Composable ColumnScope.() -> Unit,
) {
    androidx.compose.material3.ModalBottomSheet(
        onDismissRequest = onDismissRequest,
        modifier = modifier,
        sheetState = sheetState,
        scrimColor = MaterialTheme.colorScheme.onSurface.copy(alpha = alpha20),
        shape = shape,
        containerColor = containerColor,
        dragHandle = dragHandle,
        content = content
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ModalBottomSheet(
    isVisible: Boolean,
    onDismissRequest: () -> Unit,
    modifier: Modifier = Modifier,
    skipPartiallyExpanded: Boolean = false,
    containerColor: Color = MaterialTheme.colorScheme.surface,
    shape: Shape = RoundedCornerShape(topStart = sheetCornerSize, topEnd = sheetCornerSize),
    dragHandle: @Composable () -> Unit = { Box { Spacer16() } },
    content: @Composable ColumnScope.() -> Unit,
) {
    var showSheet by remember { mutableStateOf(false) }
    var presentationId by remember { mutableIntStateOf(0) }

    LaunchedEffect(isVisible) {
        if (isVisible) {
            presentationId += 1
            showSheet = true
        }
    }

    if (!showSheet) return

    key(presentationId) {
        val sheetState = rememberModalBottomSheetState(
            skipPartiallyExpanded = skipPartiallyExpanded,
        )

        LaunchedEffect(isVisible) {
            if (!isVisible) {
                if (sheetState.isVisible) {
                    sheetState.hide()
                }
                showSheet = false
            }
        }

        androidx.compose.material3.ModalBottomSheet(
            onDismissRequest = {
                showSheet = false
                onDismissRequest()
            },
            modifier = modifier,
            sheetState = sheetState,
            scrimColor = MaterialTheme.colorScheme.onSurface.copy(alpha = alpha20),
            shape = shape,
            containerColor = containerColor,
            dragHandle = dragHandle,
            content = content,
        )
    }
}
