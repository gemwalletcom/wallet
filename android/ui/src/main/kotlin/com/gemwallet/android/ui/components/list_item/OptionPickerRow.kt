package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuAnchorType
import androidx.compose.material3.ExposedDropdownMenuBox
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
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.compactIconSize
import uniffi.gemstone.GemListRow

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun <T> OptionPickerRow(row: GemListRow, listPosition: ListPosition, current: T?, options: List<T>, label: @Composable (T) -> String, onSelect: (T) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
        GemListRowView(
            row = row,
            listPosition = listPosition,
            modifier = Modifier.menuAnchor(ExposedDropdownMenuAnchorType.PrimaryNotEditable),
            onSelect = {},
        )
        ExposedDropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            matchAnchorWidth = false,
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
