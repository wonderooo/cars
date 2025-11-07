import Link from "next/link"
import { Car } from "lucide-react"

export function Footer() {
    return (
        <footer className="border-t border-border bg-card">
            <div className="container mx-auto px-4 lg:px-8 py-12">
                <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
                    {/* Brand */}
                    <div className="col-span-1 md:col-span-2">
                        <Link href="/" className="flex items-center gap-2 mb-4">
                            <Car className="h-6 w-6 text-accent" />
                            <span className="text-xl font-semibold text-foreground">VIN Clear</span>
                        </Link>
                        <p className="text-sm text-muted-foreground leading-relaxed max-w-md">
                            Professional vehicle history deletion service. We help protect your privacy and vehicle value by removing
                            records from major auction databases.
                        </p>
                    </div>

                    {/* Services */}
                    <div>
                        <h3 className="font-semibold text-foreground mb-4">Services</h3>
                        <ul className="space-y-2">
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    Copart Removal
                                </Link>
                            </li>
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    IAAI Removal
                                </Link>
                            </li>
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    VIN Check
                                </Link>
                            </li>
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    Bulk Services
                                </Link>
                            </li>
                        </ul>
                    </div>

                    {/* Company */}
                    <div>
                        <h3 className="font-semibold text-foreground mb-4">Company</h3>
                        <ul className="space-y-2">
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    About Us
                                </Link>
                            </li>
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    Contact
                                </Link>
                            </li>
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    Privacy Policy
                                </Link>
                            </li>
                            <li>
                                <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                                    Terms of Service
                                </Link>
                            </li>
                        </ul>
                    </div>
                </div>

                {/* Bottom Bar */}
                <div className="pt-8 border-t border-border flex flex-col md:flex-row justify-between items-center gap-4">
                    <p className="text-sm text-muted-foreground">© {new Date().getFullYear()} VIN Clear. All rights reserved.</p>
                    <div className="flex gap-6">
                        <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                            Privacy
                        </Link>
                        <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                            Terms
                        </Link>
                        <Link href="#" className="text-sm text-muted-foreground hover:text-foreground transition-colors">
                            Support
                        </Link>
                    </div>
                </div>
            </div>
        </footer>
    )
}
