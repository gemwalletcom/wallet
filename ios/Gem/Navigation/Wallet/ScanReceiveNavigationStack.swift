// Copyright (c). Gem Wallet. All rights reserved.

import Assets
import Primitives
import PrimitivesComponents
import QRScanner
import Recents
import Style
import SwiftUI

struct ScanReceiveNavigationStack: View {
    @State private var model: ScanReceiveViewModel

    init(model: ScanReceiveViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        NavigationStack {
            Group {
                switch model.mode {
                case .scan:
                    QRScannerScene(scanType: .universal, action: { model.onScan?($0) })
                case .receive:
                    SelectAssetScene(model: model.selectAssetModel)
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarDismissItem(
                    type: .close,
                    placement: .topBarLeading,
                )
                ToolbarItem(placement: .principal) {
                    Picker("", selection: $model.mode) {
                        ForEach(ScanReceiveMode.allCases, id: \.self) { mode in
                            Text(mode.title)
                                .tag(mode)
                        }
                    }
                    .pickerStyle(.segmented)
                    .frame(width: Sizing.picker.segmentedWidth)
                }
            }
            .onChange(of: model.selectAssetModel.assetSelection, model.onChangeAssetSelection)
        }
        .id(model.mode)
        .sheet(item: $model.isPresentingReceive) { input in
            SelectedAssetNavigationStack(
                input: input,
                wallet: model.selectAssetModel.wallet,
                onComplete: model.onCompleteReceive,
            )
        }
        .recentAssetsSheet(model: model.selectAssetModel.recentModel, onSelect: model.selectAssetModel.onSelectRecent)
    }
}
