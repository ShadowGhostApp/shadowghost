// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'bus.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AppEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AppEvent()';
}


}

/// @nodoc
class $AppEventCopyWith<$Res>  {
$AppEventCopyWith(AppEvent _, $Res Function(AppEvent) __);
}


/// Adds pattern-matching-related methods to [AppEvent].
extension AppEventPatterns on AppEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AppEvent_Network value)?  network,TResult Function( AppEvent_Storage value)?  storage,TResult Function( AppEvent_Crypto value)?  crypto,TResult Function( AppEvent_Custom value)?  custom,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AppEvent_Network() when network != null:
return network(_that);case AppEvent_Storage() when storage != null:
return storage(_that);case AppEvent_Crypto() when crypto != null:
return crypto(_that);case AppEvent_Custom() when custom != null:
return custom(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AppEvent_Network value)  network,required TResult Function( AppEvent_Storage value)  storage,required TResult Function( AppEvent_Crypto value)  crypto,required TResult Function( AppEvent_Custom value)  custom,}){
final _that = this;
switch (_that) {
case AppEvent_Network():
return network(_that);case AppEvent_Storage():
return storage(_that);case AppEvent_Crypto():
return crypto(_that);case AppEvent_Custom():
return custom(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AppEvent_Network value)?  network,TResult? Function( AppEvent_Storage value)?  storage,TResult? Function( AppEvent_Crypto value)?  crypto,TResult? Function( AppEvent_Custom value)?  custom,}){
final _that = this;
switch (_that) {
case AppEvent_Network() when network != null:
return network(_that);case AppEvent_Storage() when storage != null:
return storage(_that);case AppEvent_Crypto() when crypto != null:
return crypto(_that);case AppEvent_Custom() when custom != null:
return custom(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( NetworkEvent field0)?  network,TResult Function( StorageEvent field0)?  storage,TResult Function( CryptoEvent field0)?  crypto,TResult Function( String eventType,  String data,  BigInt timestamp)?  custom,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AppEvent_Network() when network != null:
return network(_that.field0);case AppEvent_Storage() when storage != null:
return storage(_that.field0);case AppEvent_Crypto() when crypto != null:
return crypto(_that.field0);case AppEvent_Custom() when custom != null:
return custom(_that.eventType,_that.data,_that.timestamp);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( NetworkEvent field0)  network,required TResult Function( StorageEvent field0)  storage,required TResult Function( CryptoEvent field0)  crypto,required TResult Function( String eventType,  String data,  BigInt timestamp)  custom,}) {final _that = this;
switch (_that) {
case AppEvent_Network():
return network(_that.field0);case AppEvent_Storage():
return storage(_that.field0);case AppEvent_Crypto():
return crypto(_that.field0);case AppEvent_Custom():
return custom(_that.eventType,_that.data,_that.timestamp);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( NetworkEvent field0)?  network,TResult? Function( StorageEvent field0)?  storage,TResult? Function( CryptoEvent field0)?  crypto,TResult? Function( String eventType,  String data,  BigInt timestamp)?  custom,}) {final _that = this;
switch (_that) {
case AppEvent_Network() when network != null:
return network(_that.field0);case AppEvent_Storage() when storage != null:
return storage(_that.field0);case AppEvent_Crypto() when crypto != null:
return crypto(_that.field0);case AppEvent_Custom() when custom != null:
return custom(_that.eventType,_that.data,_that.timestamp);case _:
  return null;

}
}

}

/// @nodoc


class AppEvent_Network extends AppEvent {
  const AppEvent_Network(this.field0): super._();
  

 final  NetworkEvent field0;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppEvent_NetworkCopyWith<AppEvent_Network> get copyWith => _$AppEvent_NetworkCopyWithImpl<AppEvent_Network>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppEvent_Network&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AppEvent.network(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AppEvent_NetworkCopyWith<$Res> implements $AppEventCopyWith<$Res> {
  factory $AppEvent_NetworkCopyWith(AppEvent_Network value, $Res Function(AppEvent_Network) _then) = _$AppEvent_NetworkCopyWithImpl;
@useResult
$Res call({
 NetworkEvent field0
});


$NetworkEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$AppEvent_NetworkCopyWithImpl<$Res>
    implements $AppEvent_NetworkCopyWith<$Res> {
  _$AppEvent_NetworkCopyWithImpl(this._self, this._then);

  final AppEvent_Network _self;
  final $Res Function(AppEvent_Network) _then;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AppEvent_Network(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as NetworkEvent,
  ));
}

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NetworkEventCopyWith<$Res> get field0 {
  
  return $NetworkEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class AppEvent_Storage extends AppEvent {
  const AppEvent_Storage(this.field0): super._();
  

 final  StorageEvent field0;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppEvent_StorageCopyWith<AppEvent_Storage> get copyWith => _$AppEvent_StorageCopyWithImpl<AppEvent_Storage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppEvent_Storage&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AppEvent.storage(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AppEvent_StorageCopyWith<$Res> implements $AppEventCopyWith<$Res> {
  factory $AppEvent_StorageCopyWith(AppEvent_Storage value, $Res Function(AppEvent_Storage) _then) = _$AppEvent_StorageCopyWithImpl;
@useResult
$Res call({
 StorageEvent field0
});


$StorageEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$AppEvent_StorageCopyWithImpl<$Res>
    implements $AppEvent_StorageCopyWith<$Res> {
  _$AppEvent_StorageCopyWithImpl(this._self, this._then);

  final AppEvent_Storage _self;
  final $Res Function(AppEvent_Storage) _then;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AppEvent_Storage(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as StorageEvent,
  ));
}

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$StorageEventCopyWith<$Res> get field0 {
  
  return $StorageEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class AppEvent_Crypto extends AppEvent {
  const AppEvent_Crypto(this.field0): super._();
  

 final  CryptoEvent field0;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppEvent_CryptoCopyWith<AppEvent_Crypto> get copyWith => _$AppEvent_CryptoCopyWithImpl<AppEvent_Crypto>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppEvent_Crypto&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'AppEvent.crypto(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $AppEvent_CryptoCopyWith<$Res> implements $AppEventCopyWith<$Res> {
  factory $AppEvent_CryptoCopyWith(AppEvent_Crypto value, $Res Function(AppEvent_Crypto) _then) = _$AppEvent_CryptoCopyWithImpl;
@useResult
$Res call({
 CryptoEvent field0
});


$CryptoEventCopyWith<$Res> get field0;

}
/// @nodoc
class _$AppEvent_CryptoCopyWithImpl<$Res>
    implements $AppEvent_CryptoCopyWith<$Res> {
  _$AppEvent_CryptoCopyWithImpl(this._self, this._then);

  final AppEvent_Crypto _self;
  final $Res Function(AppEvent_Crypto) _then;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(AppEvent_Crypto(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as CryptoEvent,
  ));
}

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CryptoEventCopyWith<$Res> get field0 {
  
  return $CryptoEventCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class AppEvent_Custom extends AppEvent {
  const AppEvent_Custom({required this.eventType, required this.data, required this.timestamp}): super._();
  

 final  String eventType;
 final  String data;
 final  BigInt timestamp;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppEvent_CustomCopyWith<AppEvent_Custom> get copyWith => _$AppEvent_CustomCopyWithImpl<AppEvent_Custom>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppEvent_Custom&&(identical(other.eventType, eventType) || other.eventType == eventType)&&(identical(other.data, data) || other.data == data)&&(identical(other.timestamp, timestamp) || other.timestamp == timestamp));
}


@override
int get hashCode => Object.hash(runtimeType,eventType,data,timestamp);

@override
String toString() {
  return 'AppEvent.custom(eventType: $eventType, data: $data, timestamp: $timestamp)';
}


}

/// @nodoc
abstract mixin class $AppEvent_CustomCopyWith<$Res> implements $AppEventCopyWith<$Res> {
  factory $AppEvent_CustomCopyWith(AppEvent_Custom value, $Res Function(AppEvent_Custom) _then) = _$AppEvent_CustomCopyWithImpl;
@useResult
$Res call({
 String eventType, String data, BigInt timestamp
});




}
/// @nodoc
class _$AppEvent_CustomCopyWithImpl<$Res>
    implements $AppEvent_CustomCopyWith<$Res> {
  _$AppEvent_CustomCopyWithImpl(this._self, this._then);

  final AppEvent_Custom _self;
  final $Res Function(AppEvent_Custom) _then;

/// Create a copy of AppEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? eventType = null,Object? data = null,Object? timestamp = null,}) {
  return _then(AppEvent_Custom(
eventType: null == eventType ? _self.eventType : eventType // ignore: cast_nullable_to_non_nullable
as String,data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as String,timestamp: null == timestamp ? _self.timestamp : timestamp // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc
mixin _$CryptoEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CryptoEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CryptoEvent()';
}


}

/// @nodoc
class $CryptoEventCopyWith<$Res>  {
$CryptoEventCopyWith(CryptoEvent _, $Res Function(CryptoEvent) __);
}


/// Adds pattern-matching-related methods to [CryptoEvent].
extension CryptoEventPatterns on CryptoEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CryptoEvent_KeyPairGenerated value)?  keyPairGenerated,TResult Function( CryptoEvent_Error value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CryptoEvent_KeyPairGenerated() when keyPairGenerated != null:
return keyPairGenerated(_that);case CryptoEvent_Error() when error != null:
return error(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CryptoEvent_KeyPairGenerated value)  keyPairGenerated,required TResult Function( CryptoEvent_Error value)  error,}){
final _that = this;
switch (_that) {
case CryptoEvent_KeyPairGenerated():
return keyPairGenerated(_that);case CryptoEvent_Error():
return error(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CryptoEvent_KeyPairGenerated value)?  keyPairGenerated,TResult? Function( CryptoEvent_Error value)?  error,}){
final _that = this;
switch (_that) {
case CryptoEvent_KeyPairGenerated() when keyPairGenerated != null:
return keyPairGenerated(_that);case CryptoEvent_Error() when error != null:
return error(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  keyPairGenerated,TResult Function( String error,  String operation)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CryptoEvent_KeyPairGenerated() when keyPairGenerated != null:
return keyPairGenerated();case CryptoEvent_Error() when error != null:
return error(_that.error,_that.operation);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  keyPairGenerated,required TResult Function( String error,  String operation)  error,}) {final _that = this;
switch (_that) {
case CryptoEvent_KeyPairGenerated():
return keyPairGenerated();case CryptoEvent_Error():
return error(_that.error,_that.operation);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  keyPairGenerated,TResult? Function( String error,  String operation)?  error,}) {final _that = this;
switch (_that) {
case CryptoEvent_KeyPairGenerated() when keyPairGenerated != null:
return keyPairGenerated();case CryptoEvent_Error() when error != null:
return error(_that.error,_that.operation);case _:
  return null;

}
}

}

/// @nodoc


class CryptoEvent_KeyPairGenerated extends CryptoEvent {
  const CryptoEvent_KeyPairGenerated(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CryptoEvent_KeyPairGenerated);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CryptoEvent.keyPairGenerated()';
}


}




/// @nodoc


class CryptoEvent_Error extends CryptoEvent {
  const CryptoEvent_Error({required this.error, required this.operation}): super._();
  

 final  String error;
 final  String operation;

/// Create a copy of CryptoEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CryptoEvent_ErrorCopyWith<CryptoEvent_Error> get copyWith => _$CryptoEvent_ErrorCopyWithImpl<CryptoEvent_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CryptoEvent_Error&&(identical(other.error, error) || other.error == error)&&(identical(other.operation, operation) || other.operation == operation));
}


@override
int get hashCode => Object.hash(runtimeType,error,operation);

@override
String toString() {
  return 'CryptoEvent.error(error: $error, operation: $operation)';
}


}

/// @nodoc
abstract mixin class $CryptoEvent_ErrorCopyWith<$Res> implements $CryptoEventCopyWith<$Res> {
  factory $CryptoEvent_ErrorCopyWith(CryptoEvent_Error value, $Res Function(CryptoEvent_Error) _then) = _$CryptoEvent_ErrorCopyWithImpl;
@useResult
$Res call({
 String error, String operation
});




}
/// @nodoc
class _$CryptoEvent_ErrorCopyWithImpl<$Res>
    implements $CryptoEvent_ErrorCopyWith<$Res> {
  _$CryptoEvent_ErrorCopyWithImpl(this._self, this._then);

  final CryptoEvent_Error _self;
  final $Res Function(CryptoEvent_Error) _then;

/// Create a copy of CryptoEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? error = null,Object? operation = null,}) {
  return _then(CryptoEvent_Error(
error: null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as String,operation: null == operation ? _self.operation : operation // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$NetworkEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetworkEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NetworkEvent()';
}


}

/// @nodoc
class $NetworkEventCopyWith<$Res>  {
$NetworkEventCopyWith(NetworkEvent _, $Res Function(NetworkEvent) __);
}


/// Adds pattern-matching-related methods to [NetworkEvent].
extension NetworkEventPatterns on NetworkEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NetworkEvent_ServerStarted value)?  serverStarted,TResult Function( NetworkEvent_ServerStopped value)?  serverStopped,TResult Function( NetworkEvent_MessageReceived value)?  messageReceived,TResult Function( NetworkEvent_ContactAdded value)?  contactAdded,TResult Function( NetworkEvent_Error value)?  error,TResult Function( NetworkEvent_Debug value)?  debug,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NetworkEvent_ServerStarted() when serverStarted != null:
return serverStarted(_that);case NetworkEvent_ServerStopped() when serverStopped != null:
return serverStopped(_that);case NetworkEvent_MessageReceived() when messageReceived != null:
return messageReceived(_that);case NetworkEvent_ContactAdded() when contactAdded != null:
return contactAdded(_that);case NetworkEvent_Error() when error != null:
return error(_that);case NetworkEvent_Debug() when debug != null:
return debug(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NetworkEvent_ServerStarted value)  serverStarted,required TResult Function( NetworkEvent_ServerStopped value)  serverStopped,required TResult Function( NetworkEvent_MessageReceived value)  messageReceived,required TResult Function( NetworkEvent_ContactAdded value)  contactAdded,required TResult Function( NetworkEvent_Error value)  error,required TResult Function( NetworkEvent_Debug value)  debug,}){
final _that = this;
switch (_that) {
case NetworkEvent_ServerStarted():
return serverStarted(_that);case NetworkEvent_ServerStopped():
return serverStopped(_that);case NetworkEvent_MessageReceived():
return messageReceived(_that);case NetworkEvent_ContactAdded():
return contactAdded(_that);case NetworkEvent_Error():
return error(_that);case NetworkEvent_Debug():
return debug(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NetworkEvent_ServerStarted value)?  serverStarted,TResult? Function( NetworkEvent_ServerStopped value)?  serverStopped,TResult? Function( NetworkEvent_MessageReceived value)?  messageReceived,TResult? Function( NetworkEvent_ContactAdded value)?  contactAdded,TResult? Function( NetworkEvent_Error value)?  error,TResult? Function( NetworkEvent_Debug value)?  debug,}){
final _that = this;
switch (_that) {
case NetworkEvent_ServerStarted() when serverStarted != null:
return serverStarted(_that);case NetworkEvent_ServerStopped() when serverStopped != null:
return serverStopped(_that);case NetworkEvent_MessageReceived() when messageReceived != null:
return messageReceived(_that);case NetworkEvent_ContactAdded() when contactAdded != null:
return contactAdded(_that);case NetworkEvent_Error() when error != null:
return error(_that);case NetworkEvent_Debug() when debug != null:
return debug(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int port)?  serverStarted,TResult Function()?  serverStopped,TResult Function( ChatMessage message)?  messageReceived,TResult Function( Contact contact)?  contactAdded,TResult Function( String error,  String? context)?  error,TResult Function( String message)?  debug,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NetworkEvent_ServerStarted() when serverStarted != null:
return serverStarted(_that.port);case NetworkEvent_ServerStopped() when serverStopped != null:
return serverStopped();case NetworkEvent_MessageReceived() when messageReceived != null:
return messageReceived(_that.message);case NetworkEvent_ContactAdded() when contactAdded != null:
return contactAdded(_that.contact);case NetworkEvent_Error() when error != null:
return error(_that.error,_that.context);case NetworkEvent_Debug() when debug != null:
return debug(_that.message);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int port)  serverStarted,required TResult Function()  serverStopped,required TResult Function( ChatMessage message)  messageReceived,required TResult Function( Contact contact)  contactAdded,required TResult Function( String error,  String? context)  error,required TResult Function( String message)  debug,}) {final _that = this;
switch (_that) {
case NetworkEvent_ServerStarted():
return serverStarted(_that.port);case NetworkEvent_ServerStopped():
return serverStopped();case NetworkEvent_MessageReceived():
return messageReceived(_that.message);case NetworkEvent_ContactAdded():
return contactAdded(_that.contact);case NetworkEvent_Error():
return error(_that.error,_that.context);case NetworkEvent_Debug():
return debug(_that.message);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int port)?  serverStarted,TResult? Function()?  serverStopped,TResult? Function( ChatMessage message)?  messageReceived,TResult? Function( Contact contact)?  contactAdded,TResult? Function( String error,  String? context)?  error,TResult? Function( String message)?  debug,}) {final _that = this;
switch (_that) {
case NetworkEvent_ServerStarted() when serverStarted != null:
return serverStarted(_that.port);case NetworkEvent_ServerStopped() when serverStopped != null:
return serverStopped();case NetworkEvent_MessageReceived() when messageReceived != null:
return messageReceived(_that.message);case NetworkEvent_ContactAdded() when contactAdded != null:
return contactAdded(_that.contact);case NetworkEvent_Error() when error != null:
return error(_that.error,_that.context);case NetworkEvent_Debug() when debug != null:
return debug(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class NetworkEvent_ServerStarted extends NetworkEvent {
  const NetworkEvent_ServerStarted({required this.port}): super._();
  

 final  int port;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetworkEvent_ServerStartedCopyWith<NetworkEvent_ServerStarted> get copyWith => _$NetworkEvent_ServerStartedCopyWithImpl<NetworkEvent_ServerStarted>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetworkEvent_ServerStarted&&(identical(other.port, port) || other.port == port));
}


@override
int get hashCode => Object.hash(runtimeType,port);

@override
String toString() {
  return 'NetworkEvent.serverStarted(port: $port)';
}


}

/// @nodoc
abstract mixin class $NetworkEvent_ServerStartedCopyWith<$Res> implements $NetworkEventCopyWith<$Res> {
  factory $NetworkEvent_ServerStartedCopyWith(NetworkEvent_ServerStarted value, $Res Function(NetworkEvent_ServerStarted) _then) = _$NetworkEvent_ServerStartedCopyWithImpl;
@useResult
$Res call({
 int port
});




}
/// @nodoc
class _$NetworkEvent_ServerStartedCopyWithImpl<$Res>
    implements $NetworkEvent_ServerStartedCopyWith<$Res> {
  _$NetworkEvent_ServerStartedCopyWithImpl(this._self, this._then);

  final NetworkEvent_ServerStarted _self;
  final $Res Function(NetworkEvent_ServerStarted) _then;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? port = null,}) {
  return _then(NetworkEvent_ServerStarted(
port: null == port ? _self.port : port // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class NetworkEvent_ServerStopped extends NetworkEvent {
  const NetworkEvent_ServerStopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetworkEvent_ServerStopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NetworkEvent.serverStopped()';
}


}




/// @nodoc


class NetworkEvent_MessageReceived extends NetworkEvent {
  const NetworkEvent_MessageReceived({required this.message}): super._();
  

 final  ChatMessage message;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetworkEvent_MessageReceivedCopyWith<NetworkEvent_MessageReceived> get copyWith => _$NetworkEvent_MessageReceivedCopyWithImpl<NetworkEvent_MessageReceived>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetworkEvent_MessageReceived&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'NetworkEvent.messageReceived(message: $message)';
}


}

/// @nodoc
abstract mixin class $NetworkEvent_MessageReceivedCopyWith<$Res> implements $NetworkEventCopyWith<$Res> {
  factory $NetworkEvent_MessageReceivedCopyWith(NetworkEvent_MessageReceived value, $Res Function(NetworkEvent_MessageReceived) _then) = _$NetworkEvent_MessageReceivedCopyWithImpl;
@useResult
$Res call({
 ChatMessage message
});




}
/// @nodoc
class _$NetworkEvent_MessageReceivedCopyWithImpl<$Res>
    implements $NetworkEvent_MessageReceivedCopyWith<$Res> {
  _$NetworkEvent_MessageReceivedCopyWithImpl(this._self, this._then);

  final NetworkEvent_MessageReceived _self;
  final $Res Function(NetworkEvent_MessageReceived) _then;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(NetworkEvent_MessageReceived(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as ChatMessage,
  ));
}


}

/// @nodoc


class NetworkEvent_ContactAdded extends NetworkEvent {
  const NetworkEvent_ContactAdded({required this.contact}): super._();
  

 final  Contact contact;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetworkEvent_ContactAddedCopyWith<NetworkEvent_ContactAdded> get copyWith => _$NetworkEvent_ContactAddedCopyWithImpl<NetworkEvent_ContactAdded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetworkEvent_ContactAdded&&(identical(other.contact, contact) || other.contact == contact));
}


@override
int get hashCode => Object.hash(runtimeType,contact);

@override
String toString() {
  return 'NetworkEvent.contactAdded(contact: $contact)';
}


}

/// @nodoc
abstract mixin class $NetworkEvent_ContactAddedCopyWith<$Res> implements $NetworkEventCopyWith<$Res> {
  factory $NetworkEvent_ContactAddedCopyWith(NetworkEvent_ContactAdded value, $Res Function(NetworkEvent_ContactAdded) _then) = _$NetworkEvent_ContactAddedCopyWithImpl;
@useResult
$Res call({
 Contact contact
});




}
/// @nodoc
class _$NetworkEvent_ContactAddedCopyWithImpl<$Res>
    implements $NetworkEvent_ContactAddedCopyWith<$Res> {
  _$NetworkEvent_ContactAddedCopyWithImpl(this._self, this._then);

  final NetworkEvent_ContactAdded _self;
  final $Res Function(NetworkEvent_ContactAdded) _then;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? contact = null,}) {
  return _then(NetworkEvent_ContactAdded(
contact: null == contact ? _self.contact : contact // ignore: cast_nullable_to_non_nullable
as Contact,
  ));
}


}

/// @nodoc


class NetworkEvent_Error extends NetworkEvent {
  const NetworkEvent_Error({required this.error, this.context}): super._();
  

 final  String error;
 final  String? context;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetworkEvent_ErrorCopyWith<NetworkEvent_Error> get copyWith => _$NetworkEvent_ErrorCopyWithImpl<NetworkEvent_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetworkEvent_Error&&(identical(other.error, error) || other.error == error)&&(identical(other.context, context) || other.context == context));
}


@override
int get hashCode => Object.hash(runtimeType,error,context);

@override
String toString() {
  return 'NetworkEvent.error(error: $error, context: $context)';
}


}

/// @nodoc
abstract mixin class $NetworkEvent_ErrorCopyWith<$Res> implements $NetworkEventCopyWith<$Res> {
  factory $NetworkEvent_ErrorCopyWith(NetworkEvent_Error value, $Res Function(NetworkEvent_Error) _then) = _$NetworkEvent_ErrorCopyWithImpl;
@useResult
$Res call({
 String error, String? context
});




}
/// @nodoc
class _$NetworkEvent_ErrorCopyWithImpl<$Res>
    implements $NetworkEvent_ErrorCopyWith<$Res> {
  _$NetworkEvent_ErrorCopyWithImpl(this._self, this._then);

  final NetworkEvent_Error _self;
  final $Res Function(NetworkEvent_Error) _then;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? error = null,Object? context = freezed,}) {
  return _then(NetworkEvent_Error(
error: null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as String,context: freezed == context ? _self.context : context // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class NetworkEvent_Debug extends NetworkEvent {
  const NetworkEvent_Debug({required this.message}): super._();
  

 final  String message;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NetworkEvent_DebugCopyWith<NetworkEvent_Debug> get copyWith => _$NetworkEvent_DebugCopyWithImpl<NetworkEvent_Debug>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NetworkEvent_Debug&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'NetworkEvent.debug(message: $message)';
}


}

/// @nodoc
abstract mixin class $NetworkEvent_DebugCopyWith<$Res> implements $NetworkEventCopyWith<$Res> {
  factory $NetworkEvent_DebugCopyWith(NetworkEvent_Debug value, $Res Function(NetworkEvent_Debug) _then) = _$NetworkEvent_DebugCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$NetworkEvent_DebugCopyWithImpl<$Res>
    implements $NetworkEvent_DebugCopyWith<$Res> {
  _$NetworkEvent_DebugCopyWithImpl(this._self, this._then);

  final NetworkEvent_Debug _self;
  final $Res Function(NetworkEvent_Debug) _then;

/// Create a copy of NetworkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(NetworkEvent_Debug(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$StorageEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'StorageEvent()';
}


}

/// @nodoc
class $StorageEventCopyWith<$Res>  {
$StorageEventCopyWith(StorageEvent _, $Res Function(StorageEvent) __);
}


/// Adds pattern-matching-related methods to [StorageEvent].
extension StorageEventPatterns on StorageEvent {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( StorageEvent_ContactsSaved value)?  contactsSaved,TResult Function( StorageEvent_ContactsLoaded value)?  contactsLoaded,TResult Function( StorageEvent_ChatHistorySaved value)?  chatHistorySaved,TResult Function( StorageEvent_ChatHistoryLoaded value)?  chatHistoryLoaded,TResult Function( StorageEvent_CleanupCompleted value)?  cleanupCompleted,TResult Function( StorageEvent_BackupCreated value)?  backupCreated,TResult Function( StorageEvent_Error value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case StorageEvent_ContactsSaved() when contactsSaved != null:
return contactsSaved(_that);case StorageEvent_ContactsLoaded() when contactsLoaded != null:
return contactsLoaded(_that);case StorageEvent_ChatHistorySaved() when chatHistorySaved != null:
return chatHistorySaved(_that);case StorageEvent_ChatHistoryLoaded() when chatHistoryLoaded != null:
return chatHistoryLoaded(_that);case StorageEvent_CleanupCompleted() when cleanupCompleted != null:
return cleanupCompleted(_that);case StorageEvent_BackupCreated() when backupCreated != null:
return backupCreated(_that);case StorageEvent_Error() when error != null:
return error(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( StorageEvent_ContactsSaved value)  contactsSaved,required TResult Function( StorageEvent_ContactsLoaded value)  contactsLoaded,required TResult Function( StorageEvent_ChatHistorySaved value)  chatHistorySaved,required TResult Function( StorageEvent_ChatHistoryLoaded value)  chatHistoryLoaded,required TResult Function( StorageEvent_CleanupCompleted value)  cleanupCompleted,required TResult Function( StorageEvent_BackupCreated value)  backupCreated,required TResult Function( StorageEvent_Error value)  error,}){
final _that = this;
switch (_that) {
case StorageEvent_ContactsSaved():
return contactsSaved(_that);case StorageEvent_ContactsLoaded():
return contactsLoaded(_that);case StorageEvent_ChatHistorySaved():
return chatHistorySaved(_that);case StorageEvent_ChatHistoryLoaded():
return chatHistoryLoaded(_that);case StorageEvent_CleanupCompleted():
return cleanupCompleted(_that);case StorageEvent_BackupCreated():
return backupCreated(_that);case StorageEvent_Error():
return error(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( StorageEvent_ContactsSaved value)?  contactsSaved,TResult? Function( StorageEvent_ContactsLoaded value)?  contactsLoaded,TResult? Function( StorageEvent_ChatHistorySaved value)?  chatHistorySaved,TResult? Function( StorageEvent_ChatHistoryLoaded value)?  chatHistoryLoaded,TResult? Function( StorageEvent_CleanupCompleted value)?  cleanupCompleted,TResult? Function( StorageEvent_BackupCreated value)?  backupCreated,TResult? Function( StorageEvent_Error value)?  error,}){
final _that = this;
switch (_that) {
case StorageEvent_ContactsSaved() when contactsSaved != null:
return contactsSaved(_that);case StorageEvent_ContactsLoaded() when contactsLoaded != null:
return contactsLoaded(_that);case StorageEvent_ChatHistorySaved() when chatHistorySaved != null:
return chatHistorySaved(_that);case StorageEvent_ChatHistoryLoaded() when chatHistoryLoaded != null:
return chatHistoryLoaded(_that);case StorageEvent_CleanupCompleted() when cleanupCompleted != null:
return cleanupCompleted(_that);case StorageEvent_BackupCreated() when backupCreated != null:
return backupCreated(_that);case StorageEvent_Error() when error != null:
return error(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( BigInt count)?  contactsSaved,TResult Function( BigInt count)?  contactsLoaded,TResult Function( String chatId,  BigInt messageCount)?  chatHistorySaved,TResult Function( String chatId,  BigInt messageCount)?  chatHistoryLoaded,TResult Function( BigInt removedItems)?  cleanupCompleted,TResult Function( String filePath)?  backupCreated,TResult Function( String error,  String operation)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case StorageEvent_ContactsSaved() when contactsSaved != null:
return contactsSaved(_that.count);case StorageEvent_ContactsLoaded() when contactsLoaded != null:
return contactsLoaded(_that.count);case StorageEvent_ChatHistorySaved() when chatHistorySaved != null:
return chatHistorySaved(_that.chatId,_that.messageCount);case StorageEvent_ChatHistoryLoaded() when chatHistoryLoaded != null:
return chatHistoryLoaded(_that.chatId,_that.messageCount);case StorageEvent_CleanupCompleted() when cleanupCompleted != null:
return cleanupCompleted(_that.removedItems);case StorageEvent_BackupCreated() when backupCreated != null:
return backupCreated(_that.filePath);case StorageEvent_Error() when error != null:
return error(_that.error,_that.operation);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( BigInt count)  contactsSaved,required TResult Function( BigInt count)  contactsLoaded,required TResult Function( String chatId,  BigInt messageCount)  chatHistorySaved,required TResult Function( String chatId,  BigInt messageCount)  chatHistoryLoaded,required TResult Function( BigInt removedItems)  cleanupCompleted,required TResult Function( String filePath)  backupCreated,required TResult Function( String error,  String operation)  error,}) {final _that = this;
switch (_that) {
case StorageEvent_ContactsSaved():
return contactsSaved(_that.count);case StorageEvent_ContactsLoaded():
return contactsLoaded(_that.count);case StorageEvent_ChatHistorySaved():
return chatHistorySaved(_that.chatId,_that.messageCount);case StorageEvent_ChatHistoryLoaded():
return chatHistoryLoaded(_that.chatId,_that.messageCount);case StorageEvent_CleanupCompleted():
return cleanupCompleted(_that.removedItems);case StorageEvent_BackupCreated():
return backupCreated(_that.filePath);case StorageEvent_Error():
return error(_that.error,_that.operation);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( BigInt count)?  contactsSaved,TResult? Function( BigInt count)?  contactsLoaded,TResult? Function( String chatId,  BigInt messageCount)?  chatHistorySaved,TResult? Function( String chatId,  BigInt messageCount)?  chatHistoryLoaded,TResult? Function( BigInt removedItems)?  cleanupCompleted,TResult? Function( String filePath)?  backupCreated,TResult? Function( String error,  String operation)?  error,}) {final _that = this;
switch (_that) {
case StorageEvent_ContactsSaved() when contactsSaved != null:
return contactsSaved(_that.count);case StorageEvent_ContactsLoaded() when contactsLoaded != null:
return contactsLoaded(_that.count);case StorageEvent_ChatHistorySaved() when chatHistorySaved != null:
return chatHistorySaved(_that.chatId,_that.messageCount);case StorageEvent_ChatHistoryLoaded() when chatHistoryLoaded != null:
return chatHistoryLoaded(_that.chatId,_that.messageCount);case StorageEvent_CleanupCompleted() when cleanupCompleted != null:
return cleanupCompleted(_that.removedItems);case StorageEvent_BackupCreated() when backupCreated != null:
return backupCreated(_that.filePath);case StorageEvent_Error() when error != null:
return error(_that.error,_that.operation);case _:
  return null;

}
}

}

/// @nodoc


class StorageEvent_ContactsSaved extends StorageEvent {
  const StorageEvent_ContactsSaved({required this.count}): super._();
  

 final  BigInt count;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageEvent_ContactsSavedCopyWith<StorageEvent_ContactsSaved> get copyWith => _$StorageEvent_ContactsSavedCopyWithImpl<StorageEvent_ContactsSaved>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent_ContactsSaved&&(identical(other.count, count) || other.count == count));
}


@override
int get hashCode => Object.hash(runtimeType,count);

@override
String toString() {
  return 'StorageEvent.contactsSaved(count: $count)';
}


}

/// @nodoc
abstract mixin class $StorageEvent_ContactsSavedCopyWith<$Res> implements $StorageEventCopyWith<$Res> {
  factory $StorageEvent_ContactsSavedCopyWith(StorageEvent_ContactsSaved value, $Res Function(StorageEvent_ContactsSaved) _then) = _$StorageEvent_ContactsSavedCopyWithImpl;
@useResult
$Res call({
 BigInt count
});




}
/// @nodoc
class _$StorageEvent_ContactsSavedCopyWithImpl<$Res>
    implements $StorageEvent_ContactsSavedCopyWith<$Res> {
  _$StorageEvent_ContactsSavedCopyWithImpl(this._self, this._then);

  final StorageEvent_ContactsSaved _self;
  final $Res Function(StorageEvent_ContactsSaved) _then;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? count = null,}) {
  return _then(StorageEvent_ContactsSaved(
count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class StorageEvent_ContactsLoaded extends StorageEvent {
  const StorageEvent_ContactsLoaded({required this.count}): super._();
  

 final  BigInt count;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageEvent_ContactsLoadedCopyWith<StorageEvent_ContactsLoaded> get copyWith => _$StorageEvent_ContactsLoadedCopyWithImpl<StorageEvent_ContactsLoaded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent_ContactsLoaded&&(identical(other.count, count) || other.count == count));
}


@override
int get hashCode => Object.hash(runtimeType,count);

@override
String toString() {
  return 'StorageEvent.contactsLoaded(count: $count)';
}


}

/// @nodoc
abstract mixin class $StorageEvent_ContactsLoadedCopyWith<$Res> implements $StorageEventCopyWith<$Res> {
  factory $StorageEvent_ContactsLoadedCopyWith(StorageEvent_ContactsLoaded value, $Res Function(StorageEvent_ContactsLoaded) _then) = _$StorageEvent_ContactsLoadedCopyWithImpl;
@useResult
$Res call({
 BigInt count
});




}
/// @nodoc
class _$StorageEvent_ContactsLoadedCopyWithImpl<$Res>
    implements $StorageEvent_ContactsLoadedCopyWith<$Res> {
  _$StorageEvent_ContactsLoadedCopyWithImpl(this._self, this._then);

  final StorageEvent_ContactsLoaded _self;
  final $Res Function(StorageEvent_ContactsLoaded) _then;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? count = null,}) {
  return _then(StorageEvent_ContactsLoaded(
count: null == count ? _self.count : count // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class StorageEvent_ChatHistorySaved extends StorageEvent {
  const StorageEvent_ChatHistorySaved({required this.chatId, required this.messageCount}): super._();
  

 final  String chatId;
 final  BigInt messageCount;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageEvent_ChatHistorySavedCopyWith<StorageEvent_ChatHistorySaved> get copyWith => _$StorageEvent_ChatHistorySavedCopyWithImpl<StorageEvent_ChatHistorySaved>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent_ChatHistorySaved&&(identical(other.chatId, chatId) || other.chatId == chatId)&&(identical(other.messageCount, messageCount) || other.messageCount == messageCount));
}


@override
int get hashCode => Object.hash(runtimeType,chatId,messageCount);

@override
String toString() {
  return 'StorageEvent.chatHistorySaved(chatId: $chatId, messageCount: $messageCount)';
}


}

/// @nodoc
abstract mixin class $StorageEvent_ChatHistorySavedCopyWith<$Res> implements $StorageEventCopyWith<$Res> {
  factory $StorageEvent_ChatHistorySavedCopyWith(StorageEvent_ChatHistorySaved value, $Res Function(StorageEvent_ChatHistorySaved) _then) = _$StorageEvent_ChatHistorySavedCopyWithImpl;
@useResult
$Res call({
 String chatId, BigInt messageCount
});




}
/// @nodoc
class _$StorageEvent_ChatHistorySavedCopyWithImpl<$Res>
    implements $StorageEvent_ChatHistorySavedCopyWith<$Res> {
  _$StorageEvent_ChatHistorySavedCopyWithImpl(this._self, this._then);

  final StorageEvent_ChatHistorySaved _self;
  final $Res Function(StorageEvent_ChatHistorySaved) _then;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? chatId = null,Object? messageCount = null,}) {
  return _then(StorageEvent_ChatHistorySaved(
chatId: null == chatId ? _self.chatId : chatId // ignore: cast_nullable_to_non_nullable
as String,messageCount: null == messageCount ? _self.messageCount : messageCount // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class StorageEvent_ChatHistoryLoaded extends StorageEvent {
  const StorageEvent_ChatHistoryLoaded({required this.chatId, required this.messageCount}): super._();
  

 final  String chatId;
 final  BigInt messageCount;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageEvent_ChatHistoryLoadedCopyWith<StorageEvent_ChatHistoryLoaded> get copyWith => _$StorageEvent_ChatHistoryLoadedCopyWithImpl<StorageEvent_ChatHistoryLoaded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent_ChatHistoryLoaded&&(identical(other.chatId, chatId) || other.chatId == chatId)&&(identical(other.messageCount, messageCount) || other.messageCount == messageCount));
}


@override
int get hashCode => Object.hash(runtimeType,chatId,messageCount);

@override
String toString() {
  return 'StorageEvent.chatHistoryLoaded(chatId: $chatId, messageCount: $messageCount)';
}


}

/// @nodoc
abstract mixin class $StorageEvent_ChatHistoryLoadedCopyWith<$Res> implements $StorageEventCopyWith<$Res> {
  factory $StorageEvent_ChatHistoryLoadedCopyWith(StorageEvent_ChatHistoryLoaded value, $Res Function(StorageEvent_ChatHistoryLoaded) _then) = _$StorageEvent_ChatHistoryLoadedCopyWithImpl;
@useResult
$Res call({
 String chatId, BigInt messageCount
});




}
/// @nodoc
class _$StorageEvent_ChatHistoryLoadedCopyWithImpl<$Res>
    implements $StorageEvent_ChatHistoryLoadedCopyWith<$Res> {
  _$StorageEvent_ChatHistoryLoadedCopyWithImpl(this._self, this._then);

  final StorageEvent_ChatHistoryLoaded _self;
  final $Res Function(StorageEvent_ChatHistoryLoaded) _then;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? chatId = null,Object? messageCount = null,}) {
  return _then(StorageEvent_ChatHistoryLoaded(
chatId: null == chatId ? _self.chatId : chatId // ignore: cast_nullable_to_non_nullable
as String,messageCount: null == messageCount ? _self.messageCount : messageCount // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class StorageEvent_CleanupCompleted extends StorageEvent {
  const StorageEvent_CleanupCompleted({required this.removedItems}): super._();
  

 final  BigInt removedItems;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageEvent_CleanupCompletedCopyWith<StorageEvent_CleanupCompleted> get copyWith => _$StorageEvent_CleanupCompletedCopyWithImpl<StorageEvent_CleanupCompleted>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent_CleanupCompleted&&(identical(other.removedItems, removedItems) || other.removedItems == removedItems));
}


@override
int get hashCode => Object.hash(runtimeType,removedItems);

@override
String toString() {
  return 'StorageEvent.cleanupCompleted(removedItems: $removedItems)';
}


}

/// @nodoc
abstract mixin class $StorageEvent_CleanupCompletedCopyWith<$Res> implements $StorageEventCopyWith<$Res> {
  factory $StorageEvent_CleanupCompletedCopyWith(StorageEvent_CleanupCompleted value, $Res Function(StorageEvent_CleanupCompleted) _then) = _$StorageEvent_CleanupCompletedCopyWithImpl;
@useResult
$Res call({
 BigInt removedItems
});




}
/// @nodoc
class _$StorageEvent_CleanupCompletedCopyWithImpl<$Res>
    implements $StorageEvent_CleanupCompletedCopyWith<$Res> {
  _$StorageEvent_CleanupCompletedCopyWithImpl(this._self, this._then);

  final StorageEvent_CleanupCompleted _self;
  final $Res Function(StorageEvent_CleanupCompleted) _then;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? removedItems = null,}) {
  return _then(StorageEvent_CleanupCompleted(
removedItems: null == removedItems ? _self.removedItems : removedItems // ignore: cast_nullable_to_non_nullable
as BigInt,
  ));
}


}

/// @nodoc


class StorageEvent_BackupCreated extends StorageEvent {
  const StorageEvent_BackupCreated({required this.filePath}): super._();
  

 final  String filePath;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageEvent_BackupCreatedCopyWith<StorageEvent_BackupCreated> get copyWith => _$StorageEvent_BackupCreatedCopyWithImpl<StorageEvent_BackupCreated>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent_BackupCreated&&(identical(other.filePath, filePath) || other.filePath == filePath));
}


@override
int get hashCode => Object.hash(runtimeType,filePath);

@override
String toString() {
  return 'StorageEvent.backupCreated(filePath: $filePath)';
}


}

/// @nodoc
abstract mixin class $StorageEvent_BackupCreatedCopyWith<$Res> implements $StorageEventCopyWith<$Res> {
  factory $StorageEvent_BackupCreatedCopyWith(StorageEvent_BackupCreated value, $Res Function(StorageEvent_BackupCreated) _then) = _$StorageEvent_BackupCreatedCopyWithImpl;
@useResult
$Res call({
 String filePath
});




}
/// @nodoc
class _$StorageEvent_BackupCreatedCopyWithImpl<$Res>
    implements $StorageEvent_BackupCreatedCopyWith<$Res> {
  _$StorageEvent_BackupCreatedCopyWithImpl(this._self, this._then);

  final StorageEvent_BackupCreated _self;
  final $Res Function(StorageEvent_BackupCreated) _then;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? filePath = null,}) {
  return _then(StorageEvent_BackupCreated(
filePath: null == filePath ? _self.filePath : filePath // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class StorageEvent_Error extends StorageEvent {
  const StorageEvent_Error({required this.error, required this.operation}): super._();
  

 final  String error;
 final  String operation;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$StorageEvent_ErrorCopyWith<StorageEvent_Error> get copyWith => _$StorageEvent_ErrorCopyWithImpl<StorageEvent_Error>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is StorageEvent_Error&&(identical(other.error, error) || other.error == error)&&(identical(other.operation, operation) || other.operation == operation));
}


@override
int get hashCode => Object.hash(runtimeType,error,operation);

@override
String toString() {
  return 'StorageEvent.error(error: $error, operation: $operation)';
}


}

/// @nodoc
abstract mixin class $StorageEvent_ErrorCopyWith<$Res> implements $StorageEventCopyWith<$Res> {
  factory $StorageEvent_ErrorCopyWith(StorageEvent_Error value, $Res Function(StorageEvent_Error) _then) = _$StorageEvent_ErrorCopyWithImpl;
@useResult
$Res call({
 String error, String operation
});




}
/// @nodoc
class _$StorageEvent_ErrorCopyWithImpl<$Res>
    implements $StorageEvent_ErrorCopyWith<$Res> {
  _$StorageEvent_ErrorCopyWithImpl(this._self, this._then);

  final StorageEvent_Error _self;
  final $Res Function(StorageEvent_Error) _then;

/// Create a copy of StorageEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? error = null,Object? operation = null,}) {
  return _then(StorageEvent_Error(
error: null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as String,operation: null == operation ? _self.operation : operation // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
