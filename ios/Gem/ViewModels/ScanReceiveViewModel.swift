// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Foundation
import GemstonePrimitives
import Primitives

@Observable
@MainActor
final class ScanReceiveViewModel {
    var mode: ScanReceiveMode = .scan
    var isPresentingReceive: SelectedAssetInput?

    let selectAssetModel: SelectAssetSceneViewModel

    let onScan: StringAction

    init(selectAssetModel: SelectAssetSceneViewModel, onScan: StringAction) {
        self.selectAssetModel = selectAssetModel
        self.onScan = onScan
    }
}

// MARK: - Business Logic

extension ScanReceiveViewModel {
    func onChangeRoute(_: SelectAssetRoute?, _ route: SelectAssetRoute?) {
        guard case let .asset(selection) = route else { return }
        selectAssetModel.route = nil
        isPresentingReceive = SelectedAssetInput(type: .receive(.asset), assetData: selection.assetData)
    }

    func onCompleteReceive() {
        isPresentingReceive = .none
    }
}
