package com.gemwallet.android.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.secondaryFaded
import androidx.compose.ui.draw.clip
import com.gemwallet.android.ui.theme.smallIconSize

@Composable
fun InfoButton(entity: InfoSheetEntity) {
    var showBottomSheet by remember { mutableStateOf(false) }
    Icon(
        modifier = Modifier
            .clip(RoundedCornerShape(percent = 50))
            .size(smallIconSize)
            .clickable(onClick = { showBottomSheet = true }),
        imageVector = AppIcons.InfoOutlined,
        contentDescription = "",
        tint = MaterialTheme.colorScheme.secondaryFaded,
    )
    if (showBottomSheet) {
        InfoBottomSheet(entity) {
            showBottomSheet = false
        }
    }
}
