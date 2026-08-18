// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public struct DeveloperPaymentsScene: View {
    private let onSelect: (String) -> Void

    public init(onSelect: @escaping (String) -> Void) {
        self.onSelect = onSelect
    }

    public var body: some View {
        List {
            payment("EVM Address", payload: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")
            payment("Bitcoin", payload: "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.0001")
            payment("Ethereum USDC", payload: "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48@1/transfer?address=0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326&uint256=1500000")
            payment("Solana USDC", payload: "solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=1&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
            payment("XRP", payload: "ripple:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?amount=10&dt=12345")
            payment("TON", payload: "ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA?amount=1000000000&text=order+7")
        }
        .listStyle(.insetGrouped)
        .navigationTitle("Payments")
    }

    private func payment(_ title: String, payload: String) -> some View {
        NavigationCustomLink(
            with: ListItemView(title: title),
            action: { onSelect(payload) },
        )
    }
}
