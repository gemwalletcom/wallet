package com.gemwallet.android.ui.components.dialog

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall

@Composable
fun DialogBar(
    onDismissRequest: () -> Unit,
    title: String? = null,
    dismissType: DialogBarDismissType = DialogBarDismissType.Close,
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = paddingSmall, top = paddingSmall, end = paddingSmall, bottom = 0.dp),
            contentAlignment = Alignment.Center,
        ) {
            Box(
                modifier = Modifier
                    .align(
                        when (dismissType) {
                            DialogBarDismissType.Close -> Alignment.CenterStart
                            DialogBarDismissType.Confirm -> Alignment.CenterEnd
                        }
                    )
                    .padding(horizontal = paddingHalfSmall, vertical = paddingHalfSmall),
            ) {
                IconButton(
                    onClick = onDismissRequest,
                    colors = IconButtonDefaults.iconButtonColors(
                        containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = alpha10),
                    ),
                ) {
                    Icon(
                        imageVector = when (dismissType) {
                            DialogBarDismissType.Close -> AppIcons.Close
                            DialogBarDismissType.Confirm -> AppIcons.Check
                        },
                        contentDescription = null,
                    )
                }
            }
            if (title != null) {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                )
            }
        }
    }
}

enum class DialogBarDismissType {
    Close,
    Confirm,
}
