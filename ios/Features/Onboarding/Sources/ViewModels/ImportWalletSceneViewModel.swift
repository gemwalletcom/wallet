import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemWalletImportKind
import struct Gemstone.GemWalletImportScreen
import enum Gemstone.GemWalletImportType
import protocol Gemstone.GemWalletServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import enum Gemstone.GemServiceError

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
        importScreen.title.text
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
        importScreen.showsKinds
    }

    var importTypes: [GemWalletImportKind] {
        importScreen.kinds
    }

    private var importScreen: GemWalletImportScreen {
        service.importScreen(chain: chain?.toGem())
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
        if importType.resolvesNames(), let chain {
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
            buttonState = .normal
            isPresentingAlertMessage = AlertMessage(title: alertTitle, error: error)
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
        let nameRecord = nameRecordViewModel?.state.record()
        let defaultName = try await service.defaultWalletName(chain: chain?.toGem()).text.text
        try await importWallet(
            name: service.importName(nameRecord: nameRecord, defaultName: defaultName),
            type: try service.importRequest(kind: importType, chain: chain?.toGem(), input: input, nameRecord: nameRecord),
        )
    }

    private func importWallet(name: String, type: GemWalletImportType) async throws {
        let result = try await service.importWallet(name: name, type: type, source: .import)

        let wallet = result.wallet
        await activateWallet(wallet)
        switch result {
        case .new: onComplete?(.new(wallet))
        case .existing: isPresentingExistingWalletName = wallet.name
        }
    }

    private func activateWallet(_ wallet: Wallet) async {
        preferences.acceptTerms()
        do {
            try service.setCurrentWalletId(walletId: wallet.id.id)
        } catch let error as GemServiceError {
            isPresentingAlertMessage = AlertMessage(title: alertTitle, message: error.text().text)
        } catch {
            debugLog("import wallet error: \(error)")
        }
        buttonState = .normal
    }
}
