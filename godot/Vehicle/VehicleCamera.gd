extends Camera
class_name VehicleCamera

const DISTANCE = 1.5
const APPROACH_SPEED = 4.0
const FORWARD_LOOK = 5.0

const TARGET_ANGLE = -Vector3(0, sin(PI / 8), cos(PI / 8))

var vehicle: Vehicle

func _look_at_target() -> void:
	look_at(vehicle.translation + FORWARD_LOOK * vehicle.transform.basis.xform(Vector3.FORWARD), vehicle.transform.basis.xform(Vector3.UP))

func _get_target_pos() -> Vector3:
	return vehicle.translation - DISTANCE * vehicle.transform.basis.xform(TARGET_ANGLE)

func set_initial_pos() -> void:
	translation = _get_target_pos()
	_look_at_target()

func _process(delta: float) -> void:
	# set ourselves to the proper distance
	look_at(vehicle.translation, vehicle.transform.basis.xform(Vector3.UP))
	translate_object_local(Vector3.FORWARD * (vehicle.translation.distance_to(translation) - DISTANCE))
	# find angle between us and vehicle
	translation = Vehicle._apply_approach(delta, APPROACH_SPEED, translation, _get_target_pos())
	_look_at_target()
