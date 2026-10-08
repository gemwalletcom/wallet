package com.gemwallet.android.ui.components.list_item

import android.graphics.Rect
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInWindow
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.adaptivePadding
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.menuWindowMargin
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemListRow
import kotlin.math.max

@Composable
fun <T> OptionPickerRow(row: GemListRow, listPosition: ListPosition, current: T?, options: List<T>, label: @Composable (T) -> String, onSelect: (T) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    var menuMaxHeight by remember { mutableStateOf(Dp.Unspecified) }
    val density = LocalDensity.current
    val view = LocalView.current

    Box {
        GemListRowView(row = row, listPosition = listPosition, onSelect = { expanded = true })
        Box(
            modifier = Modifier
                .matchParentSize()
                .padding(horizontal = adaptivePadding(default = paddingDefault, compact = paddingSmall)),
            contentAlignment = Alignment.CenterEnd,
        ) {
            Box(
                modifier = Modifier
                    .fillMaxHeight()
                    .onGloballyPositioned { coordinates ->
                        val visible = Rect().also(view::getWindowVisibleDisplayFrame)
                        val top = coordinates.positionInWindow().y
                        val available = max(top - visible.top, visible.bottom - top - coordinates.size.height)
                        menuMaxHeight = with(density) { (available - menuWindowMargin.toPx()).toDp() }
                    },
            ) {
                DropdownMenu(
                    expanded = expanded,
                    onDismissRequest = { expanded = false },
                    modifier = Modifier.heightIn(max = menuMaxHeight),
                    containerColor = MaterialTheme.colorScheme.background,
                ) {
                    options.forEach { option ->
                        DropdownMenuItem(
                            text = {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    if (option == current) {
                                        Icon(AppIcons.Check, null, modifier = Modifier.size(compactIconSize))
                                    } else {
                                        Spacer(modifier = Modifier.size(compactIconSize))
                                    }
                                    Spacer4()
                                    Text(label(option))
                                }
                            },
                            onClick = {
                                onSelect(option)
                                expanded = false
                            },
                        )
                    }
                }
            }
        }
    }
}
