package com.gemwallet.android.ui.components

import androidx.annotation.StringRes
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R

@Composable
fun ScanReceiveSwitcher(mode: ScanReceiveMode, onModeChange: (ScanReceiveMode) -> Unit) {
    SingleChoiceSegmentedButtonRow {
        ScanReceiveMode.entries.forEachIndexed { index, entry ->
            SegmentedButton(
                selected = entry == mode,
                onClick = { onModeChange(entry) },
                shape = SegmentedButtonDefaults.itemShape(index = index, count = ScanReceiveMode.entries.size),
            ) {
                Text(text = stringResource(id = entry.titleRes()))
            }
        }
    }
}

@StringRes
private fun ScanReceiveMode.titleRes(): Int = when (this) {
    ScanReceiveMode.Scan -> R.string.wallet_scan
    ScanReceiveMode.Receive -> R.string.wallet_receive
}
