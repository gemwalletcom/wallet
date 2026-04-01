// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Style

protocol SecurityReminderViewModel {
    var title: String { get }
    var message: String { get }
    var items: [SecurityReminderItem] { get set }
    var buttonTitle: String { get }
    var docsUrl: URL { get }

    var onNext: () -> Void { get }
}
