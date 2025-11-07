import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { MessageCircle, Shield, Clock, CheckCircle } from "lucide-react"

interface RemovalSectionProps {
    vin: string
}

export function RemovalSection({ vin }: RemovalSectionProps) {
    const whatsappNumber = "1234567890" // Replace with actual WhatsApp number
    const whatsappMessage = encodeURIComponent(`Hi, I would like to remove the history for VIN: ${vin}`)
    const whatsappLink = `https://wa.me/${whatsappNumber}?text=${whatsappMessage}`

    return (
        <div className="space-y-6 sticky top-24">
            <Card className="border-primary/20 shadow-lg">
                <CardHeader className="space-y-1">
                    <Badge className="w-fit mb-2" variant="default">
                        Professional Service
                    </Badge>
                    <CardTitle className="text-2xl">Remove This Vehicle History</CardTitle>
                    <p className="text-sm text-muted-foreground">Clean your vehicle record from Copart and IAAI databases</p>
                </CardHeader>
                <CardContent className="space-y-6">
                    {/* Benefits */}
                    <div className="space-y-3">
                        <div className="flex items-start gap-3">
                            <CheckCircle className="h-5 w-5 text-primary mt-0.5 flex-shrink-0" />
                            <div>
                                <p className="font-medium">Complete Removal</p>
                                <p className="text-sm text-muted-foreground">Remove all records from auction databases</p>
                            </div>
                        </div>
                        <div className="flex items-start gap-3">
                            <Clock className="h-5 w-5 text-primary mt-0.5 flex-shrink-0" />
                            <div>
                                <p className="font-medium">Fast Processing</p>
                                <p className="text-sm text-muted-foreground">Most requests completed within 24-48 hours</p>
                            </div>
                        </div>
                        <div className="flex items-start gap-3">
                            <Shield className="h-5 w-5 text-primary mt-0.5 flex-shrink-0" />
                            <div>
                                <p className="font-medium">Secure & Confidential</p>
                                <p className="text-sm text-muted-foreground">Your information is protected and private</p>
                            </div>
                        </div>
                    </div>

                    {/* CTA Button */}
                    <Button asChild size="lg" className="w-full text-base">
                        <a
                            href={whatsappLink}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="flex items-center justify-center gap-2"
                        >
                            <MessageCircle className="h-5 w-5" />
                            Contact Us on WhatsApp
                        </a>
                    </Button>

                    <p className="text-xs text-center text-muted-foreground">
                        Click to start a conversation about removing this VIN
                    </p>
                </CardContent>
            </Card>

            {/* Additional Info Card */}
            <Card>
                <CardHeader>
                    <CardTitle className="text-lg">How It Works</CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                    <div className="flex gap-3">
                        <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-bold flex-shrink-0">
                            1
                        </div>
                        <div>
                            <p className="font-medium text-sm">Contact Us</p>
                            <p className="text-xs text-muted-foreground">Send us a message via WhatsApp</p>
                        </div>
                    </div>
                    <div className="flex gap-3">
                        <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-bold flex-shrink-0">
                            2
                        </div>
                        <div>
                            <p className="font-medium text-sm">Verification</p>
                            <p className="text-xs text-muted-foreground">We verify your vehicle information</p>
                        </div>
                    </div>
                    <div className="flex gap-3">
                        <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-bold flex-shrink-0">
                            3
                        </div>
                        <div>
                            <p className="font-medium text-sm">Removal Process</p>
                            <p className="text-xs text-muted-foreground">We remove the records from databases</p>
                        </div>
                    </div>
                    <div className="flex gap-3">
                        <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-bold flex-shrink-0">
                            4
                        </div>
                        <div>
                            <p className="font-medium text-sm">Confirmation</p>
                            <p className="text-xs text-muted-foreground">Receive confirmation of successful removal</p>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
