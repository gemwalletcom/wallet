// Copyright (c). Gem Wallet. All rights reserved.

import XCTest

extension XCUIApplication {
    var isOnboarding: Bool {
        let create = buttons["create"].firstMatch
        let mainTab = element("mainTab")
        let launched = XCTNSPredicateExpectation(predicate: NSPredicate { _, _ in create.exists || mainTab.exists }, object: nil)
        _ = XCTWaiter().wait(for: [launched], timeout: 30)
        return create.exists
    }

    func element(_ identifier: String) -> XCUIElement {
        descendants(matching: .any)[identifier].firstMatch
    }

    func tap(_ identifier: String) {
        let element = element(identifier)
        XCTAssertTrue(element.waitForExistence(timeout: 10), "\(identifier) not found")
        element.tap()
    }

    func acceptTerms() {
        guard element("term_1").waitForExistence(timeout: 2) else {
            return
        }
        (1 ... 3).forEach { tap("term_\($0)") }
        tapMainAction()
    }

    func tapMainAction() {
        tap("main_action")
    }

    func skipEnableAuthentication(leaving identifier: String) {
        XCTAssertTrue(element(identifier).waitForNonExistence(timeout: 30), "\(identifier) still shown after saving the wallet")
        let skip = buttons["skip"].firstMatch
        if skip.waitForExistence(timeout: 2) {
            skip.tap()
        }
    }

    func tapBack() {
        navigationBars.buttons.element(boundBy: 0).tap()
    }

    func tapBack(leaving identifier: String) {
        for _ in 0 ..< 3 {
            tapBack()
            if element(identifier).waitForNonExistence(timeout: 3) {
                return
            }
        }
        XCTFail("\(identifier) still shown after going back")
    }

    func reopenFromBackground() {
        XCUIDevice.shared.press(.home)
        XCTAssertTrue(wait(for: .runningBackground, timeout: 10), "app did not enter the background")
        activate()
        XCTAssertTrue(wait(for: .runningForeground, timeout: 10), "app did not return to the foreground")
    }

    func getWords() -> [String] {
        let shown = XCTNSPredicateExpectation(predicate: NSPredicate(format: "exists == true AND label != ''"), object: staticTexts["word_11"])
        XCTAssertEqual(XCTWaiter().wait(for: [shown], timeout: 10), .completed, "word_11 not shown")
        return (0 ..< 12).map { staticTexts["word_\($0)"].label }
    }

    func pickWords(_ words: [String]) {
        let options = buttons.matching(identifier: "word_option")
        words.forEach { options.matching(NSPredicate(format: "label == %@ AND isEnabled == true", $0)).firstMatch.tap() }
    }

    func tapWalletTab() {
        tap("mainTab")
    }

    func tapWalletBar() {
        tap("walletBar")
    }

    func getWalletsCount() -> Int {
        XCTAssertTrue(element("wallet_edit").waitForExistence(timeout: 10), "wallet_edit not found")
        return descendants(matching: .any).matching(identifier: "wallet_edit").count
    }

    func tapWalletSettings() {
        tap("wallet_edit")
    }

    func logout() {
        if isOnboarding == false {
            tapWalletTab()
            tapWalletBar()
            while element("wallet_edit").waitForExistence(timeout: 2) {
                tapWalletSettings()
                buttons["Delete"].firstMatch.tap()
                alerts.buttons["Delete"].tap()
            }
        }
    }

    func tapCreateWallet() {
        buttons["Create a New Wallet"].firstMatch.tap()
    }

    func tapImportWallet() {
        let button = buttons["Import an Existing Wallet"].firstMatch
        XCTAssertTrue(button.waitForExistence(timeout: 2), "button not found")
        button.tap()
    }
}
