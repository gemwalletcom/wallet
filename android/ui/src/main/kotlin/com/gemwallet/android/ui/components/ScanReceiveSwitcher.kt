package com.gemwallet.android.ui.components

import com.gemwallet.android.ui.localization.stringRes
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.wallet.core.primitives.ScanReceiveMode

@Composable
fun ScanReceiveSwitcher(mode: ScanReceiveMode, onModeChange: (ScanReceiveMode) -> Unit) {
    SingleChoiceSegmentedButtonRow {
        ScanReceiveMode.entries.forEachIndexed { index, entry ->
            SegmentedButton(
                selected = entry == mode,
                onClick = { onModeChange(entry) },
                shape = SegmentedButtonDefaults.itemShape(index = index, count = ScanReceiveMode.entries.size),
            ) {
                Text(text = stringResource(id = entry.stringRes()))
            }
        }
    }
}
