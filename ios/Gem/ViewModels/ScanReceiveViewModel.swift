// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Foundation
import Primitives

@Observable
@MainActor
final class ScanReceiveViewModel {
    var mode: ScanReceiveMode = .scan
    var isPresentingReceive: SelectedAssetInput?

    let selectAssetModel: SelectAssetViewModel

    let onScan: StringAction

    init(selectAssetModel: SelectAssetViewModel, onScan: StringAction) {
        self.selectAssetModel = selectAssetModel
        self.onScan = onScan
    }

    var modeModels: [ScanReceiveModeViewModel] {
        ScanReceiveMode.allCases.map { ScanReceiveModeViewModel(mode: $0) }
    }
}

// MARK: - Business Logic

extension ScanReceiveViewModel {
    func onChangeAssetSelection(_: SelectAssetInput?, _ selection: SelectAssetInput?) {
        guard let selection else { return }
        selectAssetModel.assetSelection = nil
        isPresentingReceive = SelectedAssetInput(type: .receive(.asset), assetData: selection.assetData)
    }

    func onCompleteReceive() {
        isPresentingReceive = .none
    }
}
