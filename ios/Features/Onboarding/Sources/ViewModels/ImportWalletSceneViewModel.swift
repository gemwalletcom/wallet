import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemWalletImportKind
import enum Gemstone.GemWalletImportType
import protocol Gemstone.GemWalletServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Preferences

@Observable
@MainActor
final class ImportWalletSceneViewModel {
    private let service: any GemWalletServiceProtocol
    private let preferences: ObservablePreferences
    private let wordSuggester = WordSuggester()
    let type: ImportWalletType

    var input: String = ""
    var wordsSuggestion: [String] = []
    var importType: GemWalletImportKind = .phrase
    let nameRecordViewModel: NameRecordViewModel?
    var buttonState = ButtonState.normal

    var isPresentingScanner = false
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingExistingWalletName: String?

    private let onComplete: (@MainActor @Sendable (ImportWalletSceneResult) -> Void)?

    init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        nameService: any GemNameServiceProtocol,
        type: ImportWalletType,
        onComplete: (@MainActor @Sendable (ImportWalletSceneResult) -> Void)?,
    ) {
        self.service = service
        self.preferences = preferences
        self.type = type
        self.onComplete = onComplete
        nameRecordViewModel = switch type {
        case .multicoin: nil
        case .chain: NameRecordViewModel(nameService: nameService)
        }
    }

    var title: String {
        switch type {
        case .multicoin: Localized.Wallet.multicoin
        case let .chain(chain): chain.networkName
        }
    }

    var pasteButtonTitle: String {
        Localized.Common.paste
    }

    var pasteButtonImage: Image {
        Images.System.paste
    }

    var qrButtonTitle: String {
        Localized.Wallet.scan
    }

    var qrButtonImage: Image {
        Images.System.qrCodeViewfinder
    }

    var alertTitle: String {
        Localized.Errors.validation("")
    }

    var chain: Chain? {
        switch type {
        case .multicoin: .none
        case let .chain(chain): chain
        }
    }

    var showImportTypes: Bool {
        importTypes.count > 1
    }

    var importTypes: [GemWalletImportKind] {
        service.importKinds(chain: chain?.map())
    }

    var footerText: String? {
        importType.showsViewOnlyWarning() ? Localized.Wallet.importAddressWarning : nil
    }

    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }

    var shouldProtectInput: Bool {
        importType.protectsInput()
    }
}

// MARK: - Business Logic

extension ImportWalletSceneViewModel {
    func onChangeImportType(_: GemWalletImportKind, _: GemWalletImportKind) {
        input = ""
    }

    func onChangeInput(_: String, newValue: String) {
        wordsSuggestion = wordSuggester.wordSuggestionCalculate(value: newValue)
        if importType == .address, let chain {
            nameRecordViewModel?.getNameRecord(name: newValue, chain: chain)
        } else {
            nameRecordViewModel?.reset()
        }
    }

    func onSelectActionButton() async {
        buttonState = .loading(showProgress: true)

        do {
            try await importWallet()
        } catch {
            isPresentingAlertMessage = AlertMessage(
                title: alertTitle,
                message: error.localizedDescription,
            )
            buttonState = .normal
        }
    }

    func onSelectScanQR() {
        isPresentingScanner = true
    }

    func onHandleScan(_ result: String) {
        input = result
    }

    func onSelectWord(_ word: String) {
        input = wordSuggester.selectWordCalculate(
            input: input,
            word: word,
        )
    }

    func onPaste() {
        guard let string = UIPasteboard.general.string else {
            UINotificationFeedbackGenerator().notificationOccurred(.error)
            return
        }
        input = string.trim()

        if shouldProtectInput {
            CopyTypeViewModel.clearClipboard()
        }
    }

    func onSelectExistingWalletContinue() {
        onComplete?(.existing)
    }
}

// MARK: - Private

extension ImportWalletSceneViewModel {
    private func importWallet() async throws {
        let nameRecord = nameRecordViewModel?.state.result?.map()
        let defaultName = try await service.defaultWalletName(chain: chain?.map()).name
        try await importWallet(
            name: service.importName(nameRecord: nameRecord, defaultName: defaultName),
            type: try service.importRequest(kind: importType, chain: chain?.map(), input: input, nameRecord: nameRecord),
        )
    }

    private func importWallet(name: String, type: GemWalletImportType) async throws {
        let result = try await service.importWallet(name: name, type: type, source: .import)

        switch result {
        case let .new(wallet):
            await activateWallet(wallet)
            onComplete?(.new(wallet))
        case let .existing(wallet):
            await activateWallet(wallet)
            isPresentingExistingWalletName = wallet.name
        }
    }

    private func activateWallet(_ wallet: Wallet) async {
        preferences.acceptTerms()
        do {
            try service.setCurrentWalletId(walletId: wallet.id.id)
        } catch {
            isPresentingAlertMessage = AlertMessage(title: alertTitle, message: error.localizedDescription)
        }
        buttonState = .normal
    }
}
