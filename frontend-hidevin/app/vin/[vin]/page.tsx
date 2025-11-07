import {notFound} from "next/navigation"
import {Navigation} from "@/components/navigation"
import {Footer} from "@/components/footer"
import {VehicleGallery} from "@/components/vehicle-gallery"
import {VehicleInfo} from "@/components/vehicle-info"
import {RemovalSection} from "@/components/removal-section"

// Mock data - replace with actual database query
async function getVehicleByVin(vin: string) {
    // This would be replaced with actual database query
    // For now, returning mock data based on the schema
    return {
        lot_number: 12345678,
        make: "Toyota",
        model: "Camry",
        year: 2020,
        vehicle_type: "Sedan",
        vin: vin,
        estimated_retail_value: 18500,
        estimated_repair_cost: 3200,
        odometer: 45000,
        odometer_status: "Actual",
        engine_name: "2.5L I4",
        engine_cylinders: "4",
        currency: "USD",
        sale_date: new Date("2024-01-15"),
        main_damage: "Front End",
        other_damage: "Minor Scratches",
        country: "USA",
        state: "California",
        transmission: "Automatic",
        color: "Silver",
        fuel_type: "Gasoline",
        drive_type: "FWD",
        keys_status: "Present",
        sold_for: 12500,
        images: [
            {
                id: 1,
                standard_source_url: "http://localhost:9000/lot-images/81646025_1_standard",
                thumbnail_source_url: "http://localhost:9000/lot-images/81646025_1_thumbnail",
                high_res_source_url: "http://localhost:9000/lot-images/81646025_1_high-res",
                sequence_number: 1,
                image_type: "exterior",
            },
            {
                id: 2,
                standard_source_url: "/silver-toyota-camry-side-view.jpg",
                thumbnail_source_url: "/silver-toyota-camry-side-view.jpg",
                high_res_source_url: "/silver-toyota-camry-side-view.jpg",
                sequence_number: 2,
                image_type: "exterior",
            },
            {
                id: 3,
                standard_source_url: "/toyota-camry-dashboard.png",
                thumbnail_source_url: "/toyota-camry-dashboard.png",
                high_res_source_url: "/toyota-camry-dashboard.png",
                sequence_number: 3,
                image_type: "interior",
            },
            {
                id: 4,
                standard_source_url: "/toyota-camry-front-damage.jpg",
                thumbnail_source_url: "/toyota-camry-front-damage.jpg",
                high_res_source_url: "/toyota-camry-front-damage.jpg",
                sequence_number: 4,
                image_type: "damage",
            },
        ],
    }
}

export interface Vehicle {
    color: string
    country: string
    currency: string
    driveType: string | null
    engineCylinders: string | null
    engineName: string | null
    estimatedRepairCost: number
    estimatedRetailValue: number
    fuelType: string | null
    keysStatus: string | null
    lotNumber: number
    mainDamage: string
    make: string
    model: string
    odometer: number
    odometerStatus: string | null
    otherDamage: string | null
    saleDate: string
    state: string
    transmission: string | null
    vehicleType: string
    vin: string | null
    year: number
    lotImages: [VehicleImage]
}

export interface VehicleImage {
    highResBucketKey: string | null
    highResMimeType: string | null
    sequenceNumber: number
    standardBucketKey: string | null
    standardMimeType: string | null
    thumbnailBucketKey: string | null
    thumbnailMimeType: string | null
}

async function fetchVehicleByVin(vin: string): Promise<Vehicle | null> {
    const path = `http://api:8081/lot_vehicle/vin/${vin}`
    const response = await fetch(path, {
        method: 'GET',
        headers: {
            'Content-Type': 'application/json',
        },
    });
    if (!response.ok)
        return null
    return await response.json()
}

export default async function VinResultPage({
                                                params
                                            }: {
    params: Promise<{ vin: string }>
}) {
    const {vin} = await params

    const vehicle = await fetchVehicleByVin(vin)

    if (!vehicle) {
        notFound()
    }

    return (
        <div className="min-h-screen flex flex-col">
            <Navigation/>

            <main className="flex-1 pt-20">
                {/* Vehicle Title */}
                <section className="border-b">
                    <div className="container mx-auto px-4 py-8">
                        <h1 className="text-4xl font-bold mb-2">
                            {vehicle.year} {vehicle.make} {vehicle.model}
                        </h1>
                        <p className="text-muted-foreground text-lg">
                            VIN: {vehicle.vin} • Lot #{vehicle.lotNumber}
                        </p>
                    </div>
                </section>

                {/* Main Content */}
                <div className="container mx-auto px-4 py-12">
                    <div className="grid lg:grid-cols-3 gap-8">
                        {/* Left Column - Gallery and Info */}
                        <div className="lg:col-span-2 space-y-8">
                            <VehicleGallery images={vehicle.lotImages}/>
                            <VehicleInfo vehicle={vehicle}/>
                        </div>

                        {/* Right Column - Removal Section */}
                        <div className="lg:col-span-1">
                            <RemovalSection vin={vehicle.vin!}/>
                        </div>
                    </div>
                </div>
            </main>

            <Footer/>
        </div>
    )
}
