import Components
import Foundation
import func Gemstone.addressCopy
import func Gemstone.chainRow
import struct Gemstone.GemChainRow
import struct Gemstone.GemCopy
import struct Gemstone.GemReceiveAssetState
import struct Gemstone.GemReceiveNetworks
import protocol Gemstone.GemReceiveServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class ReceiveSceneViewModel: Sendable {
    var qrSize: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    private(set) var assetState: GemReceiveAssetState
    private(set) var address: String

    var presentation: ReceivePresentationType?
    var renderedImage: UIImage?
    var isPresentingAlertMessage: AlertMessage?

    private let wallet: Wallet
    private let service: any GemReceiveServiceProtocol
    private let generator = QRCodeGenerator()
    let assetQuery: ObservableQuery<AssetQueryOptional>
    private(set) var selectNetworkTask: Task<Void, Never>?

    private init(
        assetData: AssetData,
        wallet: Wallet,
        address: String,
        service: any GemReceiveServiceProtocol,
    ) {
        assetState = service.assetState(asset: assetData.asset.toGem())
        self.wallet = wallet
        self.address = address
        self.service = service
        assetQuery = ObservableQuery(AssetQueryOptional(walletId: wallet.id, assetId: assetData.asset.id), initialValue: assetData)
    }

    public convenience init(assetData: AssetData, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            assetData: assetData,
            wallet: wallet,
            address: assetData.account.address,
            service: service,
        )
    }

    public convenience init(assetAddress: AssetAddress, wallet: Wallet, service: any GemReceiveServiceProtocol) {
        self.init(
            assetData: AssetData.with(asset: assetAddress.asset, account: Account(chain: assetAddress.asset.chain, address: assetAddress.address, derivationPath: "", extendedPublicKey: nil)),
            wallet: wallet,
            address: assetAddress.address,
            service: service,
        )
    }

    var networks: GemReceiveNetworks {
        service.networks(
            asset: asset.toGem(),
            associations: (assetQuery.value?.associations ?? []).map(\.assetId.identifier),
            wallet: wallet.toGem(),
        )
    }

    func updateAsset() async {
        do {
            try await service.updateAsset(assetId: asset.id.identifier)
        } catch {
            debugLog("receive asset update error: \(error)")
        }
    }

    var title: String {
        Localized.Wallet.receive
    }

    var asset: Asset {
        assetState.asset.asset.toPrimitives()
    }

    var copyTitle: String {
        Localized.Common.copy
    }

    var warningMessage: String {
        assetState.warnings
            .map(\.text)
            .joined(separator: " ")
    }

    var copy: GemCopy {
        addressCopy(chain: asset.chain.toGem(), address: address)
    }

    var showNetworkSelector: Bool {
        networks.showsSelector
    }

    var networkSelectorModel: ReceiveNetworkSelectorViewModel {
        ReceiveNetworkSelectorViewModel(
            assetIds: networks.networks.map { AssetId(core: $0.assetId) },
        )
    }

    func chainModel(for assetId: AssetId) -> GemChainRow {
        networks.networks.first { $0.assetId == assetId.identifier }?.row ?? chainRow(chain: assetId.chain.rawValue)
    }

    var isPresentingSheet: ReceivePresentationType? {
        get {
            switch presentation {
            case .share, .networkSelector: presentation
            case .copy, nil: nil
            }
        }
        set {
            presentation = newValue
        }
    }

    var isPresentingCopyToast: Bool {
        get {
            if case .copy = presentation {
                return true
            }
            return false
        }
        set {
            presentation = newValue ? .copy : nil
        }
    }

    func activityItems(qrImage: UIImage?) -> [Any] {
        if let qrImage {
            return [qrImage, address]
        }
        return [address]
    }

    private func selectNetwork(assetId: AssetId) async {
        do {
            let asset = try await service.asset(assetId: assetId.identifier).toPrimitives()
            let account = try wallet.account(for: asset.chain)
            try Task.checkCancellation()
            assetState = service.assetState(asset: asset.toGem())
            address = account.address
        } catch {
            guard !error.isCancelled else { return }
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    private func generateQRCode() async -> UIImage? {
        await generator.generate(
            from: address,
            size: CGSize(
                width: qrSize,
                height: qrSize,
            ),
            logo: UIImage.name("logo-dark"),
        )
    }
}

// MARK: - Actions

extension ReceiveSceneViewModel {
    func onChangeAsset() async {
        do {
            try await service.enableAsset(walletId: wallet.id.id, assetId: asset.id.identifier)
        } catch {
            debugLog("ReceiveSceneViewModel enableAsset error: \(error)")
        }
    }

    func onSelectNetwork() {
        presentation = .networkSelector
    }

    func onFinishNetworkSelection(_ assetIds: [AssetId]) {
        presentation = nil
        guard let assetId = assetIds.first, assetId != asset.id else { return }

        selectNetworkTask?.cancel()
        selectNetworkTask = Task { await selectNetwork(assetId: assetId) }
    }

    func onShareSheet() {
        presentation = .share
    }

    func onCopyAddress() {
        presentation = .copy
    }

    func onLoadImage() async {
        renderedImage = nil
        renderedImage = await generateQRCode()
    }
}
