// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAutocloseSession
import SwiftUI

public struct AutocloseSheet: View {
    @State private var model: AutocloseSceneViewModel

    public init(session: GemAutocloseSession, row: GemAssetItemRow, onComplete: @escaping AutocloseCompletion) {
        _model = State(initialValue: AutocloseSceneViewModel(type: .open(session, row: row, onComplete: onComplete)))
    }

    public var body: some View {
        NavigationStack {
            AutocloseScene(model: model)
        }
    }
}
