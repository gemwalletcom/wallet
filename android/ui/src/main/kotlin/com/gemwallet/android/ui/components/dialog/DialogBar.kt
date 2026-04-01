package com.gemwallet.android.ui.components.dialog

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingDefault

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DialogBar(
    title: String? = null,
    onDismissRequest: () -> Unit,
    showDismissAction: Boolean = true,
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = paddingDefault),
    ) {
        BottomSheetDefaults.DragHandle(
            modifier = Modifier.align(Alignment.TopCenter),
            color = MaterialTheme.colorScheme.secondary.copy(alpha = 0.45f),
        )
        if (showDismissAction || title != null) {
            DialogBarHeaderContent(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = paddingDefault),
                title = title,
                showDismissAction = showDismissAction,
                onDismissRequest = onDismissRequest,
            )
        }
    }
}

@Composable
private fun DialogBarHeaderContent(
    modifier: Modifier,
    title: String?,
    showDismissAction: Boolean,
    onDismissRequest: () -> Unit,
) {
    Box(modifier = modifier) {
        if (showDismissAction) {
            DialogBarActionButton(
                modifier = Modifier.align(Alignment.CenterStart),
                onClick = onDismissRequest,
            )
        }
        title?.let {
            Text(
                text = it,
                textAlign = TextAlign.Center,
                modifier = Modifier.align(Alignment.Center),
            )
        }
    }
}

@Composable
private fun DialogBarActionButton(
    modifier: Modifier,
    onClick: () -> Unit,
) {
    ElevatedButton(
        modifier = modifier,
        shape = MaterialTheme.shapes.extraLarge,
        contentPadding = ButtonDefaults.ContentPadding,
        colors = dialogBarActionButtonColors(),
        onClick = onClick,
    ) {
        Text(
            text = stringResource(R.string.common_done),
            style = MaterialTheme.typography.labelLarge,
        )
    }
}

@Composable
private fun dialogBarActionButtonColors() = ButtonDefaults.elevatedButtonColors(
    containerColor = MaterialTheme.colorScheme.background,
    contentColor = MaterialTheme.colorScheme.primary,
    disabledContainerColor = MaterialTheme.colorScheme.background.copy(alpha = alpha10),
)
