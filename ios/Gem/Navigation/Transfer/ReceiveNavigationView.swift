// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftUI
import Transfer

struct ReceiveNavigationView: View {
    @Environment(\.viewModelFactory) private var viewModelFactory

    let assetData: AssetData
    let wallet: Wallet
    let onComplete: VoidAction

    var body: some View {
        if assetData.asset.chain == .lightning {
            LightningReceiveScene(model: viewModelFactory.lightningReceiveScene(wallet: wallet, onComplete: onComplete))
        } else {
            ReceiveScene(model: viewModelFactory.receiveScene(assetData: assetData, wallet: wallet))
        }
    }
}
