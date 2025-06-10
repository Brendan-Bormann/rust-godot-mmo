extends PlayerObject

@export var active: bool = false
@onready var nametag = $NameTag
@onready var cam = $CameraPivot/PlayerCamera
@onready var model = $model
@onready var animation_player = $model/AnimationPlayer

var last_pos = Vector2.ZERO

func lerp(a, b, t):
	return a + (b - a) * t

func _ready() -> void:
	animation_player.play("idle")

func sync_network_position():
	var lerp_speed = 0.25
	var target_pos_lerped = Vector3(lerp(position.x, network_position.x, lerp_speed), 0.0, lerp(position.z, network_position.z, lerp_speed))
	last_pos = position
	position = target_pos_lerped
	
	var move_dir = (position - last_pos).normalized()
	if move_dir.length() > 0.01:
		var target_angle = atan2(move_dir.x, move_dir.z)
		model.rotation.y = lerp_angle(model.rotation.y, target_angle, 0.2)

func is_moving():
	return (network_position - position).length() > 0.02

func _process(_delta: float) -> void:
	sync_network_position()
	nametag.text = self.username
	
	if is_moving():
		animation_player.play("run")
		var mag = (position - network_position).length()
		animation_player.speed_scale = mag * 6
	else:
		animation_player.play("idle")
		animation_player.speed_scale = 1
	
	if id == "0":
		cam.current = true
		
		var input_vector = Vector2(
			Input.get_axis("move_left", "move_right"),
			Input.get_axis("move_forward", "move_backward")
		)
		
		var rotated_vector = input_vector.rotated(PI / 4)
		GlobalNetwork.di = rotated_vector
