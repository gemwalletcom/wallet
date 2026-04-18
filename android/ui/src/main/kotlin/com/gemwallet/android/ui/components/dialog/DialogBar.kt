package com.gemwallet.android.ui.components.dialog

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.alpha50
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space4

@Composable
fun SheetHeader(
    title: String,
    onDismissRequest: () -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box(
            modifier = Modifier
                .padding(top = paddingDefault, bottom = paddingSmall)
                .width(iconSize)
                .height(space4)
                .background(
                    color = MaterialTheme.colorScheme.secondary.copy(alpha = alpha50),
                    shape = RoundedCornerShape(percent = 50),
                ),
        )
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = paddingSmall, vertical = space4),
            contentAlignment = Alignment.Center,
        ) {
            Box(modifier = Modifier.align(Alignment.CenterStart)) {
                IconButton(
                    onClick = onDismissRequest,
                    colors = IconButtonDefaults.iconButtonColors(
                        containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
                    ),
                ) {
                    Icon(imageVector = Icons.Default.Close, contentDescription = null)
                }
            }
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
            )
        }
    }
}

@Composable
fun DialogBar(
    onDismissRequest: () -> Unit,
    showDismissAction: Boolean = true,
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box(
            modifier = Modifier
                .padding(top = paddingDefault, bottom = paddingSmall)
                .width(iconSize)
                .height(space4)
                .background(
                    color = MaterialTheme.colorScheme.secondary.copy(alpha = alpha50),
                    shape = RoundedCornerShape(percent = 50),
                ),
        )
        if (showDismissAction) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = paddingDefault),
            ) {
                DialogBarActionButton(
                    modifier = Modifier.align(Alignment.CenterStart),
                    onClick = onDismissRequest,
                )
            }
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
