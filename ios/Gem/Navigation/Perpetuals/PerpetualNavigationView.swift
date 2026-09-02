import Components
import Perpetuals
import GemstoneServices
import GemstonePrimitives
import Primitives
import Store
import Style
import SwiftUI
import WalletTab

public struct PerpetualNavigationView: View {
    @State private var model: PerpetualSceneViewModel
    @Binding var isPresentingSheet: WalletSheetType?

    public init(model: PerpetualSceneViewModel, isPresentingSheet: Binding<WalletSheetType?>) {
        _isPresentingSheet = isPresentingSheet
        _model = State(initialValue: model)
    }

    public var body: some View {
        PerpetualScene(model: model)
            .sheet(isPresented: $model.isPresentingAutoclose) {
                if let position = model.positions.first {
                    AutocloseNavigationStack(
                        position: position,
                        wallet: model.wallet,
                        onComplete: model.onAutocloseComplete,
                    )
                }
            }
            .bindQuery(model.positionsQuery, model.perpetualQuery, model.transactionsQuery, model.perpetualFiatValuesQuery)
            .onChange(of: isPresentingSheet) { oldValue, newValue in
                guard newValue == nil else { return }
                switch oldValue {
                case .transferData, .perpetualRecipientData:
                    Task { await model.load() }
                default:
                    break
                }
            }
    }
}
