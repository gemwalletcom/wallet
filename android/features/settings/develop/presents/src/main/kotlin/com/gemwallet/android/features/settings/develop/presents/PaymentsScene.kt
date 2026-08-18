package com.gemwallet.android.features.settings.develop.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.screen.Scene

private val paymentOptions = listOf(
    "EVM Address" to "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326",
    "Bitcoin" to "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.0001",
    "Ethereum USDC" to "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48@1/transfer?address=0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326&uint256=1500000",
    "Solana USDC" to "solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=1&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "XRP" to "ripple:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?amount=10&dt=12345",
    "TON" to "ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA?amount=1000000000&text=order+7",
)

@Composable
fun PaymentsScene(
    onSelect: (String) -> Unit,
    onCancel: () -> Unit,
) {
    Scene(
        title = "Payments",
        onClose = onCancel,
    ) {
        LazyColumn {
            items(paymentOptions) { (title, payload) ->
                LinkItem(
                    title = title,
                    onClick = { onSelect(payload) },
                )
            }
        }
    }
}
