extends Node

@onready var online_count = $Menu/OnlineCount
@onready var connect_button = $Login/ConnectButton
@onready var login_form = $Login
@onready var username_field = $Login/Form/UsernameField
@onready var password_field = $Login/Form/PasswordField
@onready var server_addr_field = $Login/Form/ServerAddrField
@onready var menu = $Menu
@onready var fps_counter = $FPSCounter
@onready var connection_status = $ConnectionStatus

@onready var packet_counter = $PacketCounter
var last_sent = 0
var last_recv = 0
var packet_interval = 1.0
var packet_timer = 0.0

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("escape") and GlobalNetwork.active:
		toggle_menu()
	
	if GlobalNetwork.active:
		connection_status.text = "Connected"
	else:
		connection_status.text = "Disconnected"
	
	fps_counter.text = "FPS: " + str(Engine.get_frames_per_second())
	update_packet_counter(delta)

func _on_connect_button_pressed() -> void:
	var success = GlobalNetwork.connect_to_server(server_addr_field.text, username_field.text)
	
	if success:
		login_form.visible = false
		menu.visible = false

func update_packet_counter(delta: float):
	packet_timer += delta
	if packet_timer > packet_interval:
		var interval_sent = GlobalNetwork.packets_sent - last_sent
		var interval_recv = GlobalNetwork.packets_recv - last_recv
		last_sent = GlobalNetwork.packets_sent
		last_recv = GlobalNetwork.packets_recv
		packet_counter.text = "S/s: " + str(interval_sent / packet_interval) + "\nR/s: " + str(interval_recv / packet_interval)
		packet_timer = 0.0

func _on_disconnect_button_pressed() -> void:
	GlobalNetwork.disconnect()
	login_form.visible = true
	menu.visible = false

func toggle_menu():
	if menu.is_visible_in_tree():
		menu.visible = false
	else:
		menu.visible = true
