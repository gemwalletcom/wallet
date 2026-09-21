// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives
import QRScanner
import SwiftUI
import Transfer

struct RecipientNavigationView: View {
    @State private var model: RecipientSceneViewModel

    init(model: RecipientSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        RecipientScene(
            model: model,
        )
        .sheet(item: $model.isPresentingScanner) { value in
            ScanQRCodeNavigationStack(scanType: model.scanType(for: value)) {
                model.onHandleScan($0, for: value)
            }
        }
    }
}
