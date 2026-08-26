// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Foundation
import Localization
import Primitives

@Observable
@MainActor
final class ScanReceiveViewModel {
    var mode: ScanReceiveMode = .scan
    var isPresentingSheet: ScanReceiveSheetType?

    let selectAssetModel: SelectAssetViewModel

    let onScan: StringAction

    init(selectAssetModel: SelectAssetViewModel, onScan: StringAction) {
        self.selectAssetModel = selectAssetModel
        self.onScan = onScan
    }

    var modeModels: [ScanReceiveModeViewModel] {
        ScanReceiveMode.allCases.map { ScanReceiveModeViewModel(mode: $0) }
    }

    var showFilter: Bool {
        mode == .receive && selectAssetModel.showFilter
    }

    var isFilterActive: Bool {
        selectAssetModel.filterModel.isAnyFilterSpecified
    }
}

// MARK: - Business Logic

extension ScanReceiveViewModel {
    func onChangeAssetSelection(_: SelectAssetInput?, _ selection: SelectAssetInput?) {
        guard let selection else { return }
        selectAssetModel.assetSelection = nil
        isPresentingSheet = .receive(SelectedAssetInput(type: .receive(.asset), assetData: selection.assetData))
    }

    func onCompleteReceive() {
        isPresentingSheet = .none
    }

    func onSelectFilter() {
        isPresentingSheet = .filter
    }
}
